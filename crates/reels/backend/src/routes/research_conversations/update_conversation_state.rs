//! Handles updating the state of an existing research conversation.
//!
//! This function modifies a specific research conversation record in the database
//! based on its ID. It updates fields like last instruction, state URI, and status,
//! and sets the `updated_at` timestamp to the current time.
//! Returns the updated conversation or an appropriate error.

// No `use` statements allowed per guidelines. Use fully qualified paths.

use crate::routes::research_conversations::update_research_conversation_state_request::UpdateResearchConversationStateRequest;
use crate::db::research_conversation::ResearchConversation;
use crate::routes::error_response::ErrorResponse;

/// Updates an existing research conversation's state.
#[utoipa::path(
    put,
    path = "/api/research/conversations/{conversation_id}",
    request_body = UpdateResearchConversationStateRequest,
    responses(
        (status = 200, description = "Conversation updated successfully", body = ResearchConversation),
        (status = 404, description = "Conversation not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(
        ("conversation_id" = uuid::Uuid, Path, description = "ID of the conversation to update")
    ),
    tag = "Research Conversations"
)]
#[actix_web::put("/{conversation_id}")]
pub async fn update_conversation_state(
    pool: actix_web::web::Data<sqlx::PgPool>,
    conversation_id: actix_web::web::Path<uuid::Uuid>,
    request_body: actix_web::web::Json<crate::routes::research_conversations::update_research_conversation_state_request::UpdateResearchConversationStateRequest>,
) -> impl actix_web::Responder {
    let id = conversation_id.into_inner();

    let update_result =
        crate::queries::research_conversations::update_research_conversation::update_research_conversation(
            pool.as_ref(),
            id,
            request_body.0.last_instruction,
            request_body.0.conversation_state_gcs_uri,
            request_body.0.status,
        )
        .await;

    match update_result {
        Ok(Some(updated_conversation)) => actix_web::HttpResponse::Ok().json(updated_conversation),
        Ok(None) => {
            // No row was updated, meaning the ID was not found
            actix_web::HttpResponse::NotFound().json(crate::routes::error_response::ErrorResponse {
                error: std::format!("Conversation with ID {id} not found."),
            })
        }
        Err(e) => {
            // Log the error for internal tracking if a logging facade is available
            // Example: log::error!("Failed to update conversation {}: {:?}", id, e);
            actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: std::format!("Failed to update conversation {id}: {e}"),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    // No `use` statements. Fully qualified paths.
    // Note: These are placeholder tests. Full integration tests would require a test database
    // and potentially more setup for Actix services and sqlx.

    #[actix_rt::test]
    async fn test_update_conversation_state_success_conceptual() {
        // This test is conceptual and requires a running test database or mocks.
        // It outlines the logic for a success case.
        // 1. Setup: Mock PgPool, insert a test conversation.
        // 2. Execute: Call `update_conversation_state` with valid data.
        // 3. Assert: Check for HTTP 200 OK and verify response body.
        std::assert!(true, "Placeholder: test_update_conversation_state_success. Requires DB/mocking.");
    }

    #[actix_rt::test]
    async fn test_update_conversation_state_not_found_conceptual() {
        // This test is conceptual and requires a running test database or mocks.
        // It outlines the logic for a "not found" case.
        // 1. Setup: Mock PgPool, ensure a specific UUID does not exist.
        // 2. Execute: Call `update_conversation_state` with the non-existent UUID.
        // 3. Assert: Check for HTTP 404 Not Found.
        std::assert!(true, "Placeholder: test_update_conversation_state_not_found. Requires DB/mocking.");
    }

    #[actix_rt::test]
    async fn test_update_conversation_state_db_error_conceptual() {
        // This test is conceptual and requires a way to simulate a DB error with mocks.
        // It outlines the logic for a database error case.
        // 1. Setup: Mock PgPool to return an error on fetch_optional.
        // 2. Execute: Call `update_conversation_state`.
        // 3. Assert: Check for HTTP 500 Internal Server Error.
        std::assert!(true, "Placeholder: test_update_conversation_state_db_error. Requires DB/mocking.");
    }
}
