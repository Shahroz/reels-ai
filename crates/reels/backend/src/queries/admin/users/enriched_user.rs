//! Enriched user data model with personal organization credit information.
//!
//! This struct extends standard user information with credit balance data from
//! the user's personal organization. Used by admin interfaces to display user
//! credit allocations alongside basic user information. The struct implements
//! FromRow manually to work with sqlx::query (not query_as!).

use sqlx::Row;

/// Enriched user data with personal organization credit information
#[derive(Debug)]
pub struct EnrichedUser {
    pub id: uuid::Uuid,
    pub email: String,
    pub status: String,
    pub is_admin: bool,
    pub feature_flags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub credits_remaining: Option<bigdecimal::BigDecimal>,
    pub personal_org_id: Option<uuid::Uuid>,
}

impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for EnrichedUser {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(EnrichedUser {
            id: row.try_get("id")?,
            email: row.try_get("email")?,
            status: row.try_get("status")?,
            is_admin: row.try_get("is_admin")?,
            feature_flags: row.try_get("feature_flags")?,
            created_at: row.try_get("created_at")?,
            credits_remaining: row.try_get("credits_remaining")?,
            personal_org_id: row.try_get("personal_org_id")?,
        })
    }
}

