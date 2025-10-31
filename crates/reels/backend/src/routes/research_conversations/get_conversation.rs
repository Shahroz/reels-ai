//! Retrieves a specific research conversation from the database by its ID.
//!
//! This function queries the `research_conversations` table for a record
//! matching the provided `conversation_id`. Returns an Option containing
//! the conversation if found, or None otherwise.

use crate::db::research_conversation::ResearchConversation;

/// Retrieves a research conversation by its ID.
#[utoipa::path(
    get,
    path = "/api/research/conversations/{conversation_id}", // Assuming standard REST path
    responses(
        (status = 200, description = "Conversation found", body = ResearchConversation),
        (status = 404, description = "Conversation not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("conversation_id" = uuid::Uuid, Path, description = "ID of the conversation to retrieve")
        // Add other params if necessary, e.g., ("Authorization" = String, Header, description = "Bearer token")
    ),
    tag = "Research Conversations" // Assuming a tag for grouping
)]
#[actix_web::get("/{conversation_id}")]
pub async fn get_conversation(
    pool: actix_web::web::Data<sqlx::PgPool>,
    conversation_id: actix_web::web::Path<uuid::Uuid>,
) -> impl actix_web::Responder {
    let conversation_result =
        crate::queries::research_conversations::get_research_conversation_by_id::get_research_conversation_by_id(
            pool.as_ref(),
            conversation_id.into_inner(),
        )
        .await;

    match conversation_result {
        Ok(Some(conversation)) => actix_web::HttpResponse::Ok().json(conversation),
        Ok(None) => actix_web::HttpResponse::NotFound().finish(),
        Err(_e) => {
            // Consider logging the error: log::error!("Failed to get conversation {}: {:?}", conversation_id, e);
            actix_web::HttpResponse::InternalServerError().finish()
        }
    }
}
