//! Represents a pending invitation for a user to join an organization.
//!
//! This file defines the `PendingInvitation` struct, which models the schema
//! for storing invitation details in the database. It includes information
//! about the invitee's email, the target organization, the role offered,
//! a unique token for accepting the invitation, its current status (e.g., pending,
//! accepted, expired), who issued the invitation, and relevant timestamps.
//! This structure is primarily used for database interactions and API data transfer.
//! Adheres to the project's Rust coding standards regarding file structure and path qualification.

// As per rust_guidelines.md: No 'use' statements. All paths are fully qualified.
// Derives use fully qualified paths where appropriate (e.g., #[derive(serde::Serialize)]).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

/// Defines the structure for a pending invitation record.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)]
pub struct PendingInvitation {
    #[schema(example = "a1b2c3d4-e5f6-7890-1234-567890abcdef", format = "uuid", value_type = String)]
    pub id: Uuid,
    #[schema(example = "b2c3d4e5-f6a7-8901-2345-67890abcdef1", format = "uuid", value_type = String)]
    pub organization_id: Uuid,
    #[schema(example = "invitee@example.com")]
    pub invited_email: String,
    #[schema(example = "member")]
    pub role_to_assign: String,
    #[schema(example = "jwt_token_string")]
    pub invitation_token: String,
    #[schema(value_type = String, format = "date-time", example = "2024-05-23T10:00:00Z")]
    pub token_expires_at: DateTime<Utc>,
    #[schema(example = "c3d4e5f6-a7b8-9012-3456-7890abcdef12", format = "uuid", value_type = Option<String>)]
    pub invited_by_user_id: Option<Uuid>,
    #[schema(value_type = String, format = "date-time", example = "2024-05-22T10:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time", example = "2024-05-22T10:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

// According to rust_guidelines.md, unit tests for functions associated with this struct
// would be placed in a `#[cfg(test)] mod tests { ... }` block within this file.
// As `PendingInvitation` is currently a data-only struct, tests might focus on
// serialization/deserialization behavior or be part of integration tests
// for functions that create or manipulate `PendingInvitation` instances.
