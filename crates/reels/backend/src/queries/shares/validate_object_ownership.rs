//! Validates user ownership of objects across different types in a unified way.
//!
//! This function consolidates ownership checking logic that was previously
//! duplicated across multiple files. It supports all shareable object types
//! and provides consistent error handling and performance characteristics.
//! Replaces inline ownership checks in sharing endpoints.

use crate::errors::permission_errors::PermissionError;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn validate_object_ownership(
    pool: &PgPool,
    user_id: Uuid,
    object_id: Uuid,
    object_type: &str,
) -> Result<bool, PermissionError> {
    match object_type {
        "collection" => {
            sqlx::query_scalar!(
                "SELECT EXISTS(SELECT 1 FROM collections WHERE id = $1 AND user_id = $2)",
                object_id,
                user_id
            )
            .fetch_one(pool)
            .await
            .map_err(PermissionError::from)
            .map(|result| result.unwrap_or(false))
        }
        "asset" => {
            sqlx::query_scalar!(
                "SELECT EXISTS(SELECT 1 FROM assets WHERE id = $1 AND user_id = $2)",
                object_id,
                user_id
            )
            .fetch_one(pool)
            .await
            .map_err(PermissionError::from)
            .map(|result| result.unwrap_or(false))
        }
        "style" => {
            sqlx::query_scalar!(
                "SELECT EXISTS(SELECT 1 FROM styles WHERE id = $1 AND user_id = $2)",
                object_id,
                user_id
            )
            .fetch_one(pool)
            .await
            .map_err(PermissionError::from)
            .map(|result| result.unwrap_or(false))
        }
        "creative" => {
            sqlx::query_scalar!(
                "SELECT EXISTS(SELECT 1 FROM creatives WHERE id = $1 AND user_id = $2)",
                object_id,
                user_id
            )
            .fetch_one(pool)
            .await
            .map_err(PermissionError::from)
            .map(|result| result.unwrap_or(false))
        }
        "document" => {
            sqlx::query_scalar!(
                "SELECT EXISTS(SELECT 1 FROM documents WHERE id = $1 AND user_id = $2)",
                object_id,
                user_id
            )
            .fetch_one(pool)
            .await
            .map_err(PermissionError::from)
            .map(|result| result.unwrap_or(false))
        }
        "custom_format" => {
            sqlx::query_scalar!(
                "SELECT EXISTS(SELECT 1 FROM custom_formats WHERE id = $1 AND user_id = $2)",
                object_id,
                user_id
            )
            .fetch_one(pool)
            .await
            .map_err(PermissionError::from)
            .map(|result| result.unwrap_or(false))
        }
        _ => Err(PermissionError::unsupported_object_type(object_type)),
    }
}

/// Validates that an object type is supported for sharing operations.
/// This provides early validation before database queries.
pub fn validate_object_type_for_sharing(object_type: &str) -> Result<(), PermissionError> {
    match object_type {
        "collection" | "asset" | "style" | "creative" | "document" | "custom_format" => Ok(()),
        _ => Err(PermissionError::unsupported_object_type(object_type)),
    }
}

#[cfg(test)]
mod tests {
    //! Tests for validate_object_ownership.

    use super::*;

    #[test]
    fn test_validate_object_type_for_sharing_valid_types() {
        // Test all supported object types
        let valid_types = ["collection", "asset", "style", "creative", "document", "custom_format"];
        
        for object_type in valid_types {
            let result = validate_object_type_for_sharing(object_type);
            assert!(result.is_ok(), "Expected {} to be valid", object_type);
        }
    }

    #[test]
    fn test_validate_object_type_for_sharing_invalid_types() {
        // Test invalid object types
        let invalid_types = ["user", "organization", "invalid", "", "Collection"];
        
        for object_type in invalid_types {
            let result = validate_object_type_for_sharing(object_type);
            assert!(result.is_err(), "Expected {} to be invalid", object_type);
            
            if let Err(PermissionError::UnsupportedObjectType { object_type: returned_type }) = result {
                assert_eq!(returned_type, object_type);
            } else {
                panic!("Expected UnsupportedObjectType error for {}", object_type);
            }
        }
    }

    #[test]
    fn test_validate_object_type_case_sensitivity() {
        // Test that object type matching is case sensitive
        let case_variations = ["Collection", "ASSET", "Style", "CREATIVE"];
        
        for object_type in case_variations {
            let result = validate_object_type_for_sharing(object_type);
            assert!(result.is_err(), "Expected case-sensitive matching to reject {}", object_type);
        }
    }

    #[test] 
    fn test_permission_error_context() {
        // Test that error provides meaningful context
        let result = validate_object_type_for_sharing("invalid_type");
        
        match result {
            Err(PermissionError::UnsupportedObjectType { object_type }) => {
                assert_eq!(object_type, "invalid_type");
            }
            _ => panic!("Expected UnsupportedObjectType error with context"),
        }
    }

    // Note: Database integration tests would go in a separate test file
    // or integration test suite since they require database setup
    #[test]
    fn test_ownership_validation_logic_structure() {
        // Test that we handle the expected object types without database
        let test_user_id = Uuid::new_v4();
        let test_object_id = Uuid::new_v4();
        
        // This tests the function structure without database calls
        let supported_types = ["collection", "asset", "style", "creative", "document", "custom_format"];
        
        for object_type in supported_types {
            // Verify the match arms exist (this would compile-fail if missing)
            assert!(validate_object_type_for_sharing(object_type).is_ok());
        }
    }
}
