//! Defines the `UserGoogleAuth` struct, representing the schema for the `user_google_auth` database table.
//!
//! This struct stores the encrypted OAuth2 tokens for a user who has connected their Google account.
//! This data is essential for making authenticated requests to Google APIs on behalf of the user.
//! The tokens are stored as encrypted byte arrays for security.

/// Represents a user's Google OAuth credentials, stored encrypted in the database.
#[derive(std::fmt::Debug, std::clone::Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct UserGoogleAuth {
    /// The user's unique identifier, linking to the `users` table.
    #[schema(format = "uuid", value_type = String)]
    pub user_id: sqlx::types::Uuid,
    /// The encrypted OAuth2 access token.
    pub encrypted_access_token: std::vec::Vec<u8>,
    /// The encrypted OAuth2 refresh token.
    pub encrypted_refresh_token: std::vec::Vec<u8>,
    /// The expiration timestamp for the access token.
    #[schema(value_type = String, format = "date-time")]
    pub token_expiry_date: chrono::DateTime<chrono::Utc>,
    /// The timestamp when the record was created.
    #[schema(value_type = String, format = "date-time")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// The timestamp when the record was last updated.
    #[schema(value_type = String, format = "date-time")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
} 