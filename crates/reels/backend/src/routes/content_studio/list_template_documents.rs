//! Lists template documents for Content Studio.
//!
//! This endpoint retrieves documents marked as content studio templates,
//! allowing users to select templates for document transformation.
//! Supports search filtering and pagination for template management.
//! Adheres to Rust coding guidelines with FQN usage and proper error handling.

#[utoipa::path(
    get,
    path = "/api/content-studio/templates",
    tag = "Content Studio",
    params(
        ("search" = Option<String>, Query, description = "Search query to filter templates by title or content"),
        ("limit" = Option<i64>, Query, description = "Maximum number of templates to return (1-100)"),
        ("offset" = Option<i64>, Query, description = "Number of templates to skip for pagination")
    ),
    responses(
        (status = 200, description = "Template documents retrieved successfully", body = crate::routes::content_studio::responses::ListTemplateDocumentsResponse),
        (status = 400, description = "Bad request - invalid parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::get("/templates")]
pub async fn list_template_documents(
    pool: actix_web::web::Data<sqlx::PgPool>,
    auth: actix_web::web::ReqData<crate::auth::tokens::Claims>,
    query: actix_web::web::Query<crate::routes::content_studio::requests::ListTemplateDocumentsParams>,
) -> actix_web::HttpResponse {
    let user_id = auth.user_id;
    let search_pattern = if query.search.trim().is_empty() {
        String::new()
    } else {
        query.search.trim().to_string()
    };

    // Fetch template documents using our query function
    match crate::queries::documents::fetch_template_documents_for_user::fetch_template_documents_for_user(
        pool.get_ref(),
        user_id,
        &search_pattern,
        query.limit,
        query.offset,
    ).await {
        std::result::Result::Ok(templates) => {
            // Count total templates for pagination
            let total_count = match count_template_documents_for_user(
                pool.get_ref(),
                user_id,
                &search_pattern,
            ).await {
                std::result::Result::Ok(count) => count,
                std::result::Result::Err(e) => {
                    tracing::error!("Failed to count template documents for user {}: {}", user_id, e);
                    return actix_web::HttpResponse::InternalServerError()
                        .json(crate::routes::error_response::ErrorResponse::from("Failed to count templates"));
                }
            };

            let response = crate::routes::content_studio::responses::ListTemplateDocumentsResponse {
                templates: templates.clone(),
                total: total_count,
                count: templates.len() as i64,
                offset: query.offset,
            };

            tracing::info!("Retrieved {} template documents for user {}", templates.len(), user_id);
            actix_web::HttpResponse::Ok().json(response)
        }
        std::result::Result::Err(e) => {
            tracing::error!("Failed to fetch template documents for user {}: {}", user_id, e);
            actix_web::HttpResponse::InternalServerError()
                .json(crate::routes::error_response::ErrorResponse::from("Failed to retrieve templates"))
        }
    }
}

/// Counts template documents for a user with search filtering
async fn count_template_documents_for_user(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    search_pattern: &str,
) -> std::result::Result<i64, sqlx::Error> {
    let like_pattern = format!("%{search_pattern}%");
    
    let count_result = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM documents 
        WHERE (user_id = $1 OR is_public = true) 
        AND sources @> ARRAY['content_studio_template'] 
        AND (title ILIKE $2 OR content ILIKE $2)
        "#,
        user_id,
        like_pattern
    )
    .fetch_one(pool)
    .await?;

    std::result::Result::Ok(count_result.unwrap_or(0))
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_list_template_documents() {
        // Test placeholder - would implement actual endpoint test
        // Testing template listing with search and pagination
        let user_id = uuid::Uuid::new_v4();
        
        // Would test with actual web framework and database
        // assert!(list_template_documents(pool, auth, query).await.is_ok());
    }

    #[tokio::test]
    async fn test_count_template_documents_for_user() {
        // Test placeholder - would implement actual count test
        // Testing template counting with search filtering
        let user_id = uuid::Uuid::new_v4();
        let search_pattern = "";
        
        // Would test with actual database pool
        // assert!(count_template_documents_for_user(&pool, user_id, search_pattern).await.is_ok());
    }
}
