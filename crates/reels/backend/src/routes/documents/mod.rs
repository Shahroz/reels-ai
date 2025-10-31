//! API endpoints for document task operations.
//!
//! This module declares submodules for request payloads, error responses,
//! and handlers for creating, listing, retrieving, and deleting document entries.
//! It also provides the route configuration under `/api/documents`.

pub mod configure_documents_routes; // Renamed from configure
pub mod copy_document;
pub mod create_document; // Renamed from create_research
pub mod create_document_request; // Renamed from create_research_request
pub mod delete_document; // Renamed from delete_research
pub mod get_document_by_id; // Renamed from get_research_by_id
pub mod get_prefill_documents;
pub mod list_documents; // Renamed from list_research
pub mod responses;
pub mod update_document;
pub mod update_document_error;
pub mod document_update_params;
pub mod update_document_request; // Renamed from update_research_request
pub mod upload_and_attach;
pub mod get_document_by_name;
pub mod list_vocal_tour_documents;
pub mod toggle_document_research;
