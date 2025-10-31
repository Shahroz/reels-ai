//! Uploads template documents for Content Studio.
//!
//! This endpoint handles multipart file uploads and creates template documents
//! marked with the "content_studio_template" source for use in transformations.
//! Processes file content extraction and document creation with proper error handling.
//! Adheres to Rust coding guidelines with FQN usage and comprehensive validation.

#[utoipa::path(
    post,
    path = "/api/content-studio/templates/upload",
    tag = "Content Studio",
    request_body(content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Template uploaded successfully", body = crate::db::documents::Document),
        (status = 400, description = "Bad request - no file provided or invalid file"),
        (status = 401, description = "Unauthorized"),
        (status = 413, description = "File too large"),
        (status = 500, description = "Internal server error - content extraction or database failure")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::post("/templates/upload")]
pub async fn upload_template_document(
    pool: actix_web::web::Data<sqlx::PgPool>,
    auth: actix_web::web::ReqData<crate::auth::tokens::Claims>,
    mut payload: actix_multipart::Multipart,
) -> actix_web::HttpResponse {
    let user_id = auth.user_id;
    let mut uploaded_document: Option<crate::db::documents::Document> = None;
    let mut custom_title: Option<String> = None;

    while let std::result::Result::Ok(Some(mut field)) = futures_util::TryStreamExt::try_next(&mut payload).await {
        // Check if this is the custom title field
        if let Some(content_disposition) = field.content_disposition() {
            if let Some(name) = content_disposition.get_name() {
                if name == "title" {
                    // Read the custom title value
                    let mut title_data = std::vec::Vec::new();
                    while let Some(chunk) = futures_util::StreamExt::next(&mut field).await {
                        match chunk {
                            std::result::Result::Ok(data) => title_data.extend_from_slice(&data),
                            std::result::Result::Err(e) => {
                                tracing::error!("Error reading title field: {}", e);
                                break;
                            }
                        }
                    }
                    
                    if let std::result::Result::Ok(title_str) = String::from_utf8(title_data) {
                        custom_title = Some(title_str.trim().to_string());
                        tracing::debug!("Custom title received: {:?}", custom_title);
                    }
                    continue;
                }
            }
        }

        // Process file field
        let file_name = match field.content_disposition().and_then(|cd| cd.get_filename()) {
            Some(name) => name.to_string(),
            None => {
                tracing::warn!("Skipping multipart field without filename");
                continue;
            }
        };

        // Read file data
        let mut file_data: Vec<u8> = Vec::new();
        while let Some(chunk) = futures_util::StreamExt::next(&mut field).await {
            match chunk {
                std::result::Result::Ok(data) => file_data.extend_from_slice(&data),
                std::result::Result::Err(e) => {
                    tracing::error!("Error reading file stream for {}: {}", file_name, e);
                    return actix_web::HttpResponse::BadRequest()
                        .json(crate::routes::error_response::ErrorResponse::from("Error reading uploaded file"));
                }
            }
        }

        if file_data.is_empty() {
            tracing::warn!("Uploaded file is empty: {}", file_name);
            continue;
        }

        // Process the file into a template document
        let mime_type = field.content_type().map(|m| m.to_string()).unwrap_or_default();
        match process_template_file(
            pool.get_ref().clone(),
            user_id,
            file_name.clone(),
            mime_type,
            file_data,
            custom_title.clone(),
        ).await {
            std::result::Result::Ok(document) => {
                tracing::info!("Successfully created template document from file: {}", file_name);
                uploaded_document = Some(document);
                break; // Process only the first valid file
            }
            std::result::Result::Err(e) => {
                tracing::error!("Failed to process template file {}: {}", file_name, e);
                return actix_web::HttpResponse::InternalServerError()
                    .json(crate::routes::error_response::ErrorResponse::from(format!("Failed to process file: {}", e)));
            }
        }
    }

    match uploaded_document {
        Some(document) => {
            tracing::info!("Template document upload completed for user {}: {}", user_id, document.id);
            actix_web::HttpResponse::Ok().json(document)
        }
        None => {
            tracing::warn!("No valid file provided in template upload request");
            actix_web::HttpResponse::BadRequest()
                .json(crate::routes::error_response::ErrorResponse::from("No valid file provided"))
        }
    }
}

/// Processes an uploaded file into a template document
async fn process_template_file(
    pool: sqlx::PgPool,
    user_id: uuid::Uuid,
    file_name: String,
    mime_type: String,
    file_data: Vec<u8>,
    custom_title: Option<String>,
) -> std::result::Result<crate::db::documents::Document, anyhow::Error> {
    // Extract text content from the file
    let text_content = match crate::services::content_extraction::extract_text::extract_text(&file_data, &mime_type, &file_name).await {
        std::result::Result::Ok(content) => content,
        std::result::Result::Err(e) => {
            tracing::error!("Content extraction failed for template file '{}' (user {}): {}", file_name, user_id, e);
            return std::result::Result::Err(anyhow::anyhow!(e));
        }
    };
    
    // Create document title - use custom title if provided, otherwise generate from filename
    let title = match custom_title {
        Some(custom) if !custom.trim().is_empty() => custom.trim().to_string(),
        _ => format!("Template: {}", file_name),
    };
    
    // Create sources array with template marker and file origin
    let sources = vec![
        "content_studio_template".to_string(),
        format!("template_upload:{}", file_name),
    ];
    
    // Create the template document record
    let template_document = sqlx::query_as!(
        crate::db::documents::Document,
        r#"
        INSERT INTO documents (user_id, title, content, sources, status, is_public, is_task, include_research)
        VALUES ($1, $2, $3, $4, 'Completed', false, false, 'Never')
        RETURNING 
            id, user_id, title, content, sources, status, created_at, updated_at, 
            is_public, is_task, include_research as "include_research: _", collection_id
        "#,
        user_id,
        title,
        text_content,
        &sources,
    )
    .fetch_one(&pool)
    .await;

    match template_document {
        std::result::Result::Ok(doc) => {
            tracing::info!("Successfully created template document from file upload for user {}", user_id);
            std::result::Result::Ok(doc)
        },
        std::result::Result::Err(e) => {
            tracing::error!("Failed to save template document for user {}: {}", user_id, e);
            std::result::Result::Err(e.into())
        }
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_upload_template_document() {
        // Test placeholder - would implement actual upload test
        // Testing file upload and template document creation
        let user_id = uuid::Uuid::new_v4();
        
        // Would test with actual multipart payload and database
        // assert!(upload_template_document(pool, auth, payload).await.is_ok());
    }

    #[tokio::test]
    async fn test_process_template_file() {
        // Test placeholder - would implement file processing test
        // Testing content extraction and template document creation
        let user_id = uuid::Uuid::new_v4();
        let file_name = "test-template.txt".to_string();
        let mime_type = "text/plain".to_string();
        let file_data = vec![72, 101, 108, 108, 111]; // "Hello"
        
        // Would test with actual database pool
        // assert!(process_template_file(pool, user_id, file_name, mime_type, file_data, None).await.is_ok());
    }
}
