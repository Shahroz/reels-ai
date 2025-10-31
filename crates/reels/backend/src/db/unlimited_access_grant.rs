//! Database model for unlimited access grants.
//!
//! This module defines the UnlimitedAccessGrant struct that maps to the
//! unlimited_access_grants table in the database. Each grant represents
//! unlimited credit access given to either a user or an organization,
//! with full audit trail and optional expiration.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Unlimited access grant record from database
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct UnlimitedAccessGrant {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub id: Uuid,
    
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001", format = "uuid", value_type = String)]
    pub user_id: Option<Uuid>,
    
    #[schema(example = "550e8400-e29b-41d4-a716-446655440002", format = "uuid", value_type = String)]
    pub organization_id: Option<Uuid>,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub granted_at: DateTime<Utc>,
    
    #[schema(example = "550e8400-e29b-41d4-a716-446655440003", format = "uuid", value_type = String)]
    pub granted_by_user_id: Option<Uuid>,
    
    #[schema(example = "Early adopter grandfather clause")]
    pub granted_reason: String,
    
    #[schema(value_type = String, format = "date-time", example = "2024-12-31T23:59:59Z")]
    pub expires_at: Option<DateTime<Utc>>,
    
    #[schema(value_type = String, format = "date-time", example = "2024-06-15T10:00:00Z")]
    pub revoked_at: Option<DateTime<Utc>>,
    
    #[schema(example = "550e8400-e29b-41d4-a716-446655440004", format = "uuid", value_type = String)]
    pub revoked_by_user_id: Option<Uuid>,
    
    #[schema(example = "User violated terms of service")]
    pub revoked_reason: Option<String>,
    
    #[schema(example = "Additional context about this grant")]
    pub notes: Option<String>,
    
    pub metadata: Option<serde_json::Value>,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub created_at: Option<DateTime<Utc>>,
    
    #[schema(value_type = String, format = "date-time", example = "2024-01-15T10:00:00Z")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl UnlimitedAccessGrant {
    /// Check if this grant is currently active (not revoked and not expired)
    pub fn is_active(&self) -> bool {
        if self.revoked_at.is_some() {
            return false;
        }
        
        if let Some(expires_at) = self.expires_at {
            if expires_at < Utc::now() {
                return false;
            }
        }
        
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_active_grant_no_expiration() {
        let grant = UnlimitedAccessGrant {
            id: Uuid::new_v4(),
            user_id: Some(Uuid::new_v4()),
            organization_id: None,
            granted_at: Utc::now(),
            granted_by_user_id: Some(Uuid::new_v4()),
            granted_reason: String::from("Test grant"),
            expires_at: None,
            revoked_at: None,
            revoked_by_user_id: None,
            revoked_reason: None,
            notes: None,
            metadata: None,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        
        assert!(grant.is_active());
    }
    
    #[test]
    fn test_is_active_revoked_grant() {
        let grant = UnlimitedAccessGrant {
            id: Uuid::new_v4(),
            user_id: Some(Uuid::new_v4()),
            organization_id: None,
            granted_at: Utc::now(),
            granted_by_user_id: Some(Uuid::new_v4()),
            granted_reason: String::from("Test grant"),
            expires_at: None,
            revoked_at: Some(Utc::now()),
            revoked_by_user_id: Some(Uuid::new_v4()),
            revoked_reason: Some(String::from("Test revocation")),
            notes: None,
            metadata: None,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        
        assert!(!grant.is_active());
    }
    
    #[test]
    fn test_is_active_expired_grant() {
        let grant = UnlimitedAccessGrant {
            id: Uuid::new_v4(),
            user_id: Some(Uuid::new_v4()),
            organization_id: None,
            granted_at: Utc::now(),
            granted_by_user_id: Some(Uuid::new_v4()),
            granted_reason: String::from("Test grant"),
            expires_at: Some(Utc::now() - chrono::Duration::days(1)),
            revoked_at: None,
            revoked_by_user_id: None,
            revoked_reason: None,
            notes: None,
            metadata: None,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };
        
        assert!(!grant.is_active());
    }
}

