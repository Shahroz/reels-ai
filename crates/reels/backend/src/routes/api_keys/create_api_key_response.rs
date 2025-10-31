//! Defines the response structure for creating a new API key.
//!
//! This struct holds the raw API key generated upon successful creation.
//! It is serialized to JSON for the API response.
//! Conforms to the coding standards by being the sole item in this file.
//! Uses `serde` for serialization and `utoipa` for schema generation.

/// Response containing the newly generated raw API key.
#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateApiKeyResponse {
    pub raw_key: std::string::String, // Use fully qualified String
}
