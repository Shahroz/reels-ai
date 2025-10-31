//! Content Studio routes module.
//!
//! This module provides API endpoints for the content studio functionality,
//! including document transformation, content generation, and studio journey management.

pub mod transform_document;
pub mod generate_content;
pub mod get_document_lineage;
pub mod create_document_journey;
pub mod add_document_to_journey;
pub mod get_document_journey;
pub mod get_journey_by_id;
pub mod delete_document;
pub mod list_template_documents;
pub mod upload_template_document;
pub mod responses;
pub mod requests;
pub mod configure_content_studio_routes;
