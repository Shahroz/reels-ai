//! Represents an organization entity in the database.
//!
//! This file defines the `Organization` struct, mirroring the `organizations` table schema.
//! Query functions are located in `crate::queries::organizations`.
//! Adheres to the project's Rust coding standards.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, FromRow};
use utoipa::ToSchema;

/// Represents an organization, typically a company or team.
/// Can be a personal organization (one per user) or a team organization.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, ToSchema)]
pub struct Organization {
    #[schema(example = "a1b2c3d4-e5f6-7890-1234-567890abcdef", format = "uuid", value_type = String)]
    pub id: Uuid,
    #[schema(example = "Acme Corporation")]
    pub name: String,
    #[schema(example = "b2c3d4e5-f6a7-8901-2345-67890abcdef1", format = "uuid", value_type = String)]
    pub owner_user_id: Uuid,
    #[schema(value_type = Option<String>, example = "cus_test_customer_123")]
    pub stripe_customer_id: Option<String>,
    #[schema(value_type = Option<Object>, example = json!({"theme": "dark"}))]
    pub settings: Option<serde_json::Value>,
    #[schema(example = "false")]
    pub is_personal: bool,
    #[schema(value_type = String, format = "date-time", example = "2024-05-05T10:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time", example = "2024-05-05T12:00:00Z")]
    pub updated_at: DateTime<Utc>,
}
