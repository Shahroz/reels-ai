//! Request context utilities for custom events.
//!
//! This module provides utilities to extract request context from HTTP requests
//! for custom analytics events. This maintains compatibility with the previous
//! middleware event structure while being specifically designed for custom events.

/// Request data structure for custom events (simplified from middleware version)
#[derive(Debug, Clone)]
pub struct RequestData {
    pub method: String,
    pub path: String,
    pub full_url: String,
    pub query_string: String,
    pub headers: std::collections::HashMap<String, String>,
    pub query_params: serde_json::Value,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub real_ip: Option<String>,
    pub forwarded_for: Option<String>,
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
    pub http_version: String,
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
    pub content_encoding: Option<String>,
    pub accept_language: Option<String>,
    pub accept_encoding: Option<String>,
    pub request_body: Option<String>,
    pub request_body_size: Option<u64>,
    pub request_body_truncated: bool,
    pub user_registration_date: Option<chrono::NaiveDate>,
    pub cookies: std::collections::HashMap<String, String>,
    pub request_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub user_id: Option<uuid::Uuid>,
    pub session_id: Option<String>,
} 