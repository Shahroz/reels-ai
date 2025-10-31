//! Permission-specific error types for collection sharing functionality.
//!
//! This module provides structured error handling for permission resolution,
//! batch permission checking, and collection hierarchy operations. It uses
//! the thiserror crate for better error ergonomics and provides specific
//! error contexts for debugging and user feedback.

#[derive(thiserror::Error, Debug)]
pub enum PermissionError {
    #[error("Database error during permission check: {source}")]
    DatabaseError {
        #[from]
        source: sqlx::Error,
    },

    #[error("Invalid batch size: {size}. Maximum allowed: {max_allowed}")]
    BatchSizeExceeded { size: usize, max_allowed: usize },

    #[error("Collection hierarchy lookup failed for collection {collection_id}: {reason}")]
    HierarchyLookupFailed {
        collection_id: uuid::Uuid,
        reason: String,
    },

    #[error("Permission resolution failed for object {object_id}: {reason}")]
    PermissionResolutionFailed {
        object_id: uuid::Uuid,
        reason: String,
    },

    #[error("Invalid object type for sharing: {object_type}")]
    UnsupportedObjectType { object_type: String },

    #[error("Access denied: insufficient permissions for object {object_id}")]
    AccessDenied { object_id: uuid::Uuid },

    #[error("Ownership check failed for object {object_id} of type {object_type}")]
    OwnershipCheckFailed {
        object_id: uuid::Uuid,
        object_type: String,
    },
}

impl PermissionError {
    pub fn batch_size_exceeded(size: usize, max_allowed: usize) -> Self {
        Self::BatchSizeExceeded { size, max_allowed }
    }

    pub fn hierarchy_lookup_failed(collection_id: uuid::Uuid, reason: impl Into<String>) -> Self {
        Self::HierarchyLookupFailed {
            collection_id,
            reason: reason.into(),
        }
    }

    pub fn permission_resolution_failed(object_id: uuid::Uuid, reason: impl Into<String>) -> Self {
        Self::PermissionResolutionFailed {
            object_id,
            reason: reason.into(),
        }
    }

    pub fn unsupported_object_type(object_type: impl Into<String>) -> Self {
        Self::UnsupportedObjectType {
            object_type: object_type.into(),
        }
    }

    pub fn access_denied(object_id: uuid::Uuid) -> Self {
        Self::AccessDenied { object_id }
    }

    pub fn ownership_check_failed(object_id: uuid::Uuid, object_type: impl Into<String>) -> Self {
        Self::OwnershipCheckFailed {
            object_id,
            object_type: object_type.into(),
        }
    }
}

// Convert PermissionError to HTTP responses for API endpoints
impl From<PermissionError> for actix_web::HttpResponse {
    fn from(error: PermissionError) -> Self {
        use actix_web::HttpResponse;
        use crate::routes::error_response::ErrorResponse;

        match error {
            PermissionError::DatabaseError { .. } => {
                log::error!("Database error in permission check: {}", error);
                HttpResponse::InternalServerError().json(ErrorResponse::from("Permission check failed"))
            }
            PermissionError::BatchSizeExceeded { size, max_allowed } => {
                log::warn!("Batch size exceeded: {} > {}", size, max_allowed);
                HttpResponse::BadRequest().json(ErrorResponse::from(format!(
                    "Batch size {} exceeds maximum allowed {}",
                    size, max_allowed
                )))
            }
            PermissionError::UnsupportedObjectType { ref object_type } => {
                log::warn!("Unsupported object type: {}", object_type);
                HttpResponse::BadRequest().json(ErrorResponse::from(format!(
                    "Unsupported object type: {}",
                    object_type
                )))
            }
            PermissionError::AccessDenied { object_id } => {
                log::info!("Access denied for object: {}", object_id);
                HttpResponse::Forbidden().json(ErrorResponse::from("Access denied"))
            }
            PermissionError::HierarchyLookupFailed { collection_id, ref reason } => {
                log::error!("Hierarchy lookup failed for {}: {}", collection_id, reason);
                HttpResponse::InternalServerError().json(ErrorResponse::from("Collection access check failed"))
            }
            PermissionError::PermissionResolutionFailed { object_id, ref reason } => {
                log::error!("Permission resolution failed for {}: {}", object_id, reason);
                HttpResponse::InternalServerError().json(ErrorResponse::from("Permission resolution failed"))
            }
            PermissionError::OwnershipCheckFailed { object_id, ref object_type } => {
                log::error!("Ownership check failed for {} ({})", object_id, object_type);
                HttpResponse::InternalServerError().json(ErrorResponse::from("Ownership verification failed"))
            }
        }
    }
}
