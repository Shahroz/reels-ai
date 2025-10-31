//! This module provides a service for interacting with the Google Drive API.
//!
//! It encapsulates all the logic for OAuth2 authentication, token management,
//! and file operations like fetching metadata and downloading content. It also
//! handles API-specific error handling and retries.

pub mod download_file_content;
pub mod exchange_code_for_tokens;
pub mod get_authorization_url;
pub mod get_file_metadata;
pub mod models;
pub mod refresh_access_token;
pub mod revoke_token; 