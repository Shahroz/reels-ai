//! Configures the routes for research tasks under `/api/research`.
//!
//! This module defines the scope and registers handlers for research endpoints:
//! `create_document`, `list_documents`, `get_document_by_id`, `update_document`, `delete_document`, and `copy_document`.

use crate::routes::documents::copy_document::copy_document;
use crate::routes::documents::create_document::create_document;
use crate::routes::documents::delete_document::delete_document;
use crate::routes::documents::get_document_by_id::get_document_by_id;
use crate::routes::documents::get_prefill_documents::get_prefill_documents;
use crate::routes::documents::list_documents::list_documents;
use crate::routes::documents::update_document::update_document;
use crate::routes::documents::upload_and_attach::upload_and_attach;
use crate::routes::documents::get_document_by_name::get_document_by_name;
use crate::routes::documents::list_vocal_tour_documents::list_vocal_tour_documents;
use crate::routes::documents::toggle_document_research::toggle_document_research;
use actix_web::web;

/// Configures all routes for the documents feature.
pub fn configure_documents_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(get_prefill_documents)
            .service(create_document)
            .service(list_documents)
            .service(list_vocal_tour_documents)  // Move before get_document_by_id to avoid route conflict
            .service(get_document_by_name)
            .service(get_document_by_id)  // This {id} route must come after specific routes
            .service(update_document)
            .service(delete_document)
            .service(copy_document)
            .service(upload_and_attach)
            .service(toggle_document_research)
    );
}
