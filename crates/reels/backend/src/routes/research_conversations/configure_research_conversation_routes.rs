//! Configures Actix-web routes for research conversation operations.
//!
//! This function is responsible for registering all HTTP handlers
//! related to research conversations, such as creating, retrieving,
//! and managing conversation records. It adheres to the project's
//! routing structure and Rust coding guidelines.

use crate::routes::research_conversations::create_conversation::create_conversation;
use crate::routes::research_conversations::delete_conversation::delete_conversation;
use crate::routes::research_conversations::get_conversation::get_conversation;
use crate::routes::research_conversations::list_conversations_by_document::list_conversations_by_document;
use crate::routes::research_conversations::update_conversation_state::update_conversation_state;

/// Configures research conversation-specific routes.
///
/// This function adds the necessary services and handlers to the Actix-web
/// application configuration for the `/api/research/conversations` endpoint.
pub fn configure_research_conversation_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("") // Base path for this scope will be mounted e.g. /api/research/conversations
            .service(create_conversation) // Handles POST to the base path
            .service(get_conversation)    // Handles GET to /{conversation_id}
            .service(update_conversation_state) // Handles PUT to /{conversation_id}
            .service(delete_conversation) // Handles DELETE to /{conversation_id}
            .service(list_conversations_by_document) // Handles GET to /by-document/{document_id}
    );
}