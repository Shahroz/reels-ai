//! Handles the creation of a new research conversation record in the database.
//!
//! This function takes the necessary data, inserts a new row into the
//! `research_conversations` table, and returns the newly created conversation object.
//! Uses fully qualified paths as per guidelines.

use crate::db::research_conversation::ResearchConversation;
use crate::routes::research_conversations::create_research_conversation_request::CreateResearchConversationRequest;

/// Creates a new research conversation record.
#[utoipa::path(
    post,
    path = "/api/research/conversations", // Assuming standard REST path
    request_body = CreateResearchConversationRequest,
    responses(
        (status = 201, description = "Conversation created successfully", body = ResearchConversation),
        (status = 500, description = "Internal server error")
    ),
    params(
        // Add params if necessary, e.g., ("Authorization" = String, Header, description = "Bearer token")
    ),
    tag = "Research Conversations" // Assuming a tag for grouping
)]
#[actix_web::post("")]
pub async fn create_conversation(
    pool: actix_web::web::Data<sqlx::PgPool>,
    data: actix_web::web::Json<crate::routes::research_conversations::create_research_conversation_request::CreateResearchConversationRequest>,
) -> impl actix_web::Responder {
    let conversation_result =
        crate::queries::research_conversations::create_research_conversation::create_research_conversation(
            pool.as_ref(),
            data.0.user_id,
            data.0.document_id,
            data.0.last_instruction.clone(),
            data.0.status.clone(),
        )
        .await;

    match conversation_result {
        Ok(conversation) => actix_web::HttpResponse::Created().json(conversation), // Use 201 Created for POST
        Err(_e) => {
            // Consider logging the error: log::error!("Failed to create conversation: {:?}", e);
            actix_web::HttpResponse::InternalServerError().finish()
        }
    }
}
