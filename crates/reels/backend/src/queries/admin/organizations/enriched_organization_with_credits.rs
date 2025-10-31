//! Enriched organization data model with credit and ownership information.
//!
//! This struct extends standard organization information with owner details, member count,
//! and credit balance data. Used by admin interfaces to display comprehensive organization
//! information including credit allocations. The struct implements FromRow manually to work
//! with sqlx::query (not query_as!).

use sqlx::Row;

/// Enriched organization data with owner and credit information
#[derive(Debug)]
pub struct EnrichedOrganizationWithCredits {
    pub id: uuid::Uuid,
    pub name: String,
    pub owner_user_id: uuid::Uuid,
    pub owner_email: String,
    pub member_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub credits_remaining: Option<bigdecimal::BigDecimal>,
}

impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for EnrichedOrganizationWithCredits {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(EnrichedOrganizationWithCredits {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            owner_user_id: row.try_get("owner_user_id")?,
            owner_email: row.try_get("owner_email")?,
            member_count: row.try_get("member_count")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            credits_remaining: row.try_get("credits_remaining")?,
        })
    }
}

