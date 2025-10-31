//! Configuration for content studio routes.

use actix_web::web;

use super::transform_document::transform_document;
use super::generate_content::generate_content;
use super::get_document_lineage::get_document_lineage;
use super::create_document_journey::create_document_journey;
use super::add_document_to_journey::add_document_to_journey;
use super::get_document_journey::get_document_journey_by_document_id;
use super::get_journey_by_id::get_journey_by_id;
use super::delete_document::delete_content_studio_document;
use super::list_template_documents::list_template_documents;
use super::upload_template_document::upload_template_document;

/// Configure content studio routes
pub fn configure_content_studio_routes(cfg: &mut web::ServiceConfig) {
    log::info!("Configuring Content Studio routes");
    cfg.service(
        web::scope("")
            .service(transform_document)
            .service(generate_content)
            .service(create_document_journey)
            .service(add_document_to_journey)
            .service(get_document_journey_by_document_id)
            .service(get_journey_by_id)
            .service(delete_content_studio_document)
            .service(list_template_documents)
            .service(upload_template_document)
            .route("/document-lineage/{document_id}", web::get().to(get_document_lineage))
    );
    log::info!("Content Studio routes configured successfully");
}
