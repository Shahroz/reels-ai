//! Module for document-related database query functions.
//!
//! This module aggregates specific query operations for the `documents` entity.
//! It adheres to the one-item-per-file and FQN guidelines.
//! Organizes functions like counting, fetching, etc., for documents.

pub mod count_documents_for_user;
pub mod count_public_documents;
pub mod count_user_documents;
pub mod delete_document_entry;
pub mod fetch_documents_for_user;
pub mod find_document_by_id_and_user;
pub mod insert_document_entry;
pub mod update_document_entry;

pub mod fetch_always_include_documents;
pub mod fetch_public_documents;
pub mod fetch_user_documents;
pub mod fetch_template_documents_for_user;

pub mod check_update_permissions;
pub mod fetch_document_access_details;
pub mod fetch_document_with_access_info;
pub mod fetch_document_for_copy;
pub mod fetch_user_email;
pub mod fetch_user_organization_ids;
pub mod fetch_user_organization_ids_from_pool;
pub mod insert_document_copy;
pub mod fetch_document_for_copy_from_pool;
pub mod fetch_user_email_from_pool;
pub mod lineage;
