//! Represents an organization membership entity in the database.
//!
//! This file defines the `OrganizationMember` struct and `OrganizationMemberStatus` enum,
//! mirroring the `organization_members` table schema. It links users to organizations
//! and defines their role and status. Query functions are now located in
//! `crate::queries::organizations`.
//!
//! Adheres to the project's Rust coding standards.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, FromRow};
use std::str::FromStr; // For OrganizationMemberStatus
use utoipa::ToSchema;

/// Enum for organization member statuses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum OrganizationMemberStatus {
    Active,
    Invited,
    Rejected,
    // Add other statuses as needed, e.g., Inactive
}

impl ToString for OrganizationMemberStatus {
    fn to_string(&self) -> String {
        match self {
            OrganizationMemberStatus::Active => "active".to_string(),
            OrganizationMemberStatus::Invited => "invited".to_string(),
            OrganizationMemberStatus::Rejected => "rejected".to_string(),
        }
    }
}

impl FromStr for OrganizationMemberStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(OrganizationMemberStatus::Active),
            "invited" => Ok(OrganizationMemberStatus::Invited),
            "rejected" => Ok(OrganizationMemberStatus::Rejected),
            _ => Err(format!("'{s}' is not a valid organization member status")),
        }
    }
}

/// Represents a user's membership within an organization.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)]
pub struct OrganizationMember {
    #[schema(example = "a1b2c3d4-e5f6-7890-1234-567890abcdef", format = "uuid", value_type = String)]
    pub organization_id: Uuid,
    #[schema(example = "c3d4e5f6-a7b8-9012-3456-7890abcdef12", format = "uuid", value_type = String)]
    pub user_id: Uuid,
    #[schema(example = "admin")]
    pub role: String, // e.g., 'admin', 'member'
    #[schema(example = "active")]
    pub status: String, // e.g., 'active', 'invited', 'inactive'
    #[schema(example = "d4e5f6a7-b8c9-0123-4567-890abcdef123", format = "uuid", value_type = Option<String>)]
    pub invited_by_user_id: Option<Uuid>,
    #[schema(value_type = Option<String>, format = "date-time", example = "2024-05-05T09:00:00Z")]
    pub invited_at: Option<DateTime<Utc>>,
    #[schema(value_type = Option<String>, format = "date-time", example = "2024-05-05T11:00:00Z")]
    pub joined_at: Option<DateTime<Utc>>,
}