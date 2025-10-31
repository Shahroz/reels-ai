//! Defines the data structures for interacting with the Google Drive API.
//!
//! This module contains structs that correspond to the JSON responses from
//! various Google API endpoints, such as the token endpoint and the files endpoint.
//! These are used with `serde` to deserialize the API responses.

/// Represents the successful response from Google's OAuth2 token endpoint.
#[derive(serde::Deserialize, std::fmt::Debug)]
pub struct GoogleTokenResponse {
    pub access_token: std::string::String,
    pub expires_in: i64,
    pub refresh_token: Option<std::string::String>,
    pub scope: std::string::String,
    pub token_type: std::string::String,
}

/// Represents the metadata for a file in Google Drive.
#[derive(serde::Deserialize, std::fmt::Debug)]
pub struct GoogleFileMetadata {
    pub id: std::string::String,
    pub name: std::string::String,
    #[serde(rename = "mimeType")]
    pub mime_type: std::string::String,
    pub version: std::string::String,
} 