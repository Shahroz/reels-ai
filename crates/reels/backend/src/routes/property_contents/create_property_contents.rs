//! Handler for creating marketing content from property description.
//!
//! This endpoint takes a source document ID, extracts the property description,
//! calls the GenNodes PropertyDescriptionToContents workflow, and creates multiple
//! marketing documents for different platforms. Generated documents inherit the
//! collection_id from the source document to maintain listing association.

use actix_web::post;

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreatePropertyContentsRequest {
    /// ID of the source document containing property description
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub source_document_id: uuid::Uuid,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct CreatePropertyContentsResponse {
    /// List of created marketing documents
    pub documents: std::vec::Vec<crate::db::documents::Document>,
    /// Total number of documents created
    pub total_documents: usize,
    /// Source document that was used for generation
    pub source_document: crate::db::documents::Document,
}

#[utoipa::path(
    post,
    path = "/api/property-contents",
    tag = "Property Contents",
    request_body = CreatePropertyContentsRequest,
    responses(
        (status = 200, description = "Marketing content created successfully", body = CreatePropertyContentsResponse),
        (status = 400, description = "Bad Request - Invalid document ID or missing property description"),
        (status = 404, description = "Source document not found or access denied"),
        (status = 500, description = "Internal Server Error - Content generation failed")
    ),
    security(("user_auth" = []))
)]
#[post("")]
pub async fn create_property_contents(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
    req: actix_web::web::Json<CreatePropertyContentsRequest>,
) -> impl actix_web::Responder {
    let user_id = claims.user_id;
    let source_document_id = req.source_document_id;

    log::info!("Creating property contents for user {} from document {}", user_id, source_document_id);

    match execute_property_contents_workflow(&pool, user_id, source_document_id).await {
        std::result::Result::Ok(response) => {
            log::info!("Successfully created {} marketing documents for user {} from document {}", 
                       response.total_documents, user_id, source_document_id);
            actix_web::HttpResponse::Ok().json(response)
        }
        std::result::Result::Err(e) => {
            log::error!("Property contents generation failed for user {} and document {}: {}", user_id, source_document_id, e);
            
            if e.contains("not found") || e.contains("access denied") {
                actix_web::HttpResponse::NotFound().json(
                    crate::routes::assets::error_response::ErrorResponse {
                        error: e,
                    }
                )
            } else if e.contains("Insufficient content") || e.contains("Invalid document") {
                actix_web::HttpResponse::BadRequest().json(
                    crate::routes::assets::error_response::ErrorResponse {
                        error: e,
                    }
                )
            } else {
                actix_web::HttpResponse::InternalServerError().json(
                    crate::routes::assets::error_response::ErrorResponse {
                        error: "Property contents generation failed".into(),
                    }
                )
            }
        }
    }
}

/// Executes the property contents workflow orchestration logic.
/// 
/// This function contains the core step-by-step logic of the property contents workflow:
/// 1. Validates user ownership and fetches source document with collection_id
/// 2. Extracts property description from document content
/// 3. Calls the PropertyDescriptionToContents GenNodes workflow
/// 4. Parses the response into structured marketing content items
/// 5. Creates formatted documents with inherited collection_id
/// 
/// # Arguments
/// 
/// * `pool` - Database connection pool
/// * `user_id` - ID of the authenticated user
/// * `source_document_id` - ID of the source document containing property description
/// 
/// # Returns
/// 
/// A `Result` containing the `CreatePropertyContentsResponse` on success, or an error message.
pub async fn execute_property_contents_workflow(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    source_document_id: uuid::Uuid,
) -> std::result::Result<CreatePropertyContentsResponse, std::string::String> {
    // Step 1: Validate user ownership and fetch source document with collection_id
    log::info!("Fetching source document {} for user {}", source_document_id, user_id);
    
    let source_document = match crate::queries::documents::find_document_by_id_and_user::find_document_by_id_and_user(
        pool, 
        source_document_id, 
        user_id
    ).await {
        std::result::Result::Ok(std::option::Option::Some(doc)) => doc,
        std::result::Result::Ok(std::option::Option::None) => {
            return std::result::Result::Err("Source document not found or access denied".to_string());
        }
        std::result::Result::Err(e) => {
            return std::result::Result::Err(format!("Failed to fetch source document: {}", e));
        }
    };

    // Extract collection_id for inheritance
    let source_collection_id = source_document.collection_id;
    log::info!("Source document has collection_id: {:?}", source_collection_id);

    // Step 2: Extract property description from document content
    let property_description = match extract_property_description(&source_document.content) {
        std::result::Result::Ok(desc) => desc,
        std::result::Result::Err(e) => {
            return std::result::Result::Err(e);
        }
    };

    log::info!("Extracted property description: {} characters", property_description.len());

    // Step 3: Call PropertyDescriptionToContents GenNodes workflow
    let params = crate::agent_tools::tool_params::property_description_to_contents_params::PropertyDescriptionToContentsParams {
        property_info: property_description,
        user_id: std::option::Option::Some(user_id),
    };

    let (full_response, _user_response) = match crate::agent_tools::handlers::handle_property_description_to_contents::handle_property_description_to_contents(params).await {
        std::result::Result::Ok(responses) => responses,
        std::result::Result::Err(e) => {
            return std::result::Result::Err(format!("GenNodes workflow failed: {}", e));
        }
    };

    log::info!("GenNodes workflow completed successfully");

    // Step 4: Parse GenNodes response into marketing content items with collection inheritance
    let marketing_contents = match crate::routes::property_contents::parse_property_marketing_response::parse_property_marketing_response(
        &full_response.response, 
        source_collection_id
    ) {
        std::result::Result::Ok(contents) => contents,
        std::result::Result::Err(e) => {
            return std::result::Result::Err(format!("Failed to parse GenNodes response: {}", e));
        }
    };

    log::info!("Parsed {} marketing content items", marketing_contents.len());

    // Step 5: Create documents for each marketing content item
    let mut created_documents = std::vec::Vec::new();
    
    let mut tx = match pool.begin().await {
        std::result::Result::Ok(tx) => tx,
        std::result::Result::Err(e) => {
            return std::result::Result::Err(format!("Failed to start database transaction: {}", e));
        }
    };

    for marketing_content in &marketing_contents {
        // Format content using appropriate template
        let formatted_html = crate::routes::property_contents::property_content_templates::format_marketing_content(marketing_content);

        // Create source reference to the original document
        let sources = vec![source_document_id.to_string()];

        // Insert document with collection_id inheritance
                  let document_data = match crate::queries::documents::insert_document_entry::insert_document_entry(
            &mut tx,
            std::option::Option::Some(user_id),
            &marketing_content.title,
            &formatted_html,
            &sources,
            false, // is_public
            false, // is_task
            std::option::Option::Some(crate::db::document_research_usage::DocumentResearchUsage::Never),
            std::option::Option::None, // No collection_id for property contents
        ).await {
            std::result::Result::Ok(data) => data,
            std::result::Result::Err(e) => {
                let _ = tx.rollback().await;
                return std::result::Result::Err(format!("Failed to create document '{}': {}", marketing_content.title, e));
            }
        };

        // Update the document with collection_id if source had one
        if let std::option::Option::Some(collection_id) = marketing_content.collection_id {
            match sqlx::query!(
                "UPDATE documents SET collection_id = $1, updated_at = NOW() WHERE id = $2",
                collection_id,
                document_data.id
            )
            .execute(&mut *tx)
            .await {
                std::result::Result::Ok(_) => {
                    log::debug!("Set collection_id {} for document {}", collection_id, document_data.id);
                }
                std::result::Result::Err(e) => {
                    let _ = tx.rollback().await;
                    return std::result::Result::Err(format!("Failed to set collection_id for document '{}': {}", marketing_content.title, e));
                }
            }
        }

        // Convert InsertedDocumentData to Document
        let document = crate::db::documents::Document {
            id: document_data.id,
            user_id: document_data.user_id,
            title: document_data.title,
            content: document_data.content,
            sources: document_data.sources,
            status: document_data.status,
            created_at: document_data.created_at,
            updated_at: document_data.updated_at,
            is_public: document_data.is_public,
            is_task: document_data.is_task,
            include_research: document_data.include_research.and_then(|s| s.parse().ok()),
            collection_id: marketing_content.collection_id,
        };

        created_documents.push(document);
        log::debug!("Created document: {} ({})", marketing_content.title, marketing_content.content_type);
    }

    // Commit transaction
    if let std::result::Result::Err(e) = tx.commit().await {
        return std::result::Result::Err(format!("Failed to commit document creation transaction: {}", e));
    }

    log::info!("Successfully created {} marketing documents", created_documents.len());

    // Step 6: Return response
    let response = CreatePropertyContentsResponse {
        total_documents: created_documents.len(),
        documents: created_documents,
        source_document,
    };

    std::result::Result::Ok(response)
}

/// Extracts property description from document content.
/// 
/// Performs basic HTML stripping and content validation to ensure the document
/// contains sufficient information for marketing content generation.
/// 
/// # Arguments
/// 
/// * `content` - The raw document content (may contain HTML)
/// 
/// # Returns
/// 
/// A cleaned property description string, or an error if insufficient content.
pub fn extract_property_description(content: &str) -> std::result::Result<std::string::String, std::string::String> {
    // Basic HTML tag removal - strip common HTML tags
    let text_content = content
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("</p>", "\n")
        .replace("</div>", "\n")
        .replace("</h1>", "\n")
        .replace("</h2>", "\n")
        .replace("</h3>", "\n")
        .replace("</h4>", "\n")
        .replace("</h5>", "\n")
        .replace("</h6>", "\n")
        .replace("</li>", "\n");

    // Remove HTML tags using a simple regex-like approach
    let mut cleaned = std::string::String::new();
    let mut in_tag = false;
    
    for char in text_content.chars() {
        match char {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => cleaned.push(char),
            _ => {} // Skip characters inside tags
        }
    }

    // Clean up whitespace and decode HTML entities
    let cleaned = cleaned
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");

    // Normalize whitespace
    let words: std::vec::Vec<&str> = cleaned.split_whitespace().collect();
    let normalized = words.join(" ");

    // Validate content length
    if normalized.len() < 50 {
        return std::result::Result::Err(
            "Insufficient content: Document must contain at least 50 characters of property description".to_string()
        );
    }

    if normalized.len() > 10000 {
        // Truncate to reasonable length for GenNodes processing
        let truncated = normalized.chars().take(10000).collect::<std::string::String>();
        log::warn!("Truncated property description from {} to {} characters", normalized.len(), truncated.len());
        return std::result::Result::Ok(truncated);
    }

    std::result::Result::Ok(normalized)
} 