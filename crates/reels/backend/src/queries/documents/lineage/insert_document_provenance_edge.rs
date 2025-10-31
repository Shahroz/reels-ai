//! Document-specific provenance edge creation.
//! 
//! Wrapper around the existing provenance system specifically for documents.

use sqlx::PgPool;
use uuid::Uuid;

use crate::queries::provenance::insert_provenance_edge::{insert_provenance_edge, NodeRef};
use super::types::DocumentDerivationParams;

/// Insert a provenance edge between two documents
pub async fn insert_document_provenance_edge(
    pool: &PgPool,
    source_document_id: Uuid,
    target_document_id: Uuid,
    derivation_params: DocumentDerivationParams,
) -> Result<(), sqlx::Error> {
    let source = NodeRef::Document(source_document_id);
    let target = NodeRef::Document(target_document_id);
    
    let (relation_type, params) = match derivation_params {
        DocumentDerivationParams::ContentTransformation(transformation_params) => {
            let params = serde_json::json!({
                "transformation_prompt": transformation_params.transformation_prompt
            });
            ("content_transformation", params)
        }
        DocumentDerivationParams::ContentGeneration => {
            let params = serde_json::json!({});
            ("content_generation", params)
        }
    };
    
    insert_provenance_edge(
        pool,
        &source,
        &target,
        relation_type,
        &params,
    ).await
}
