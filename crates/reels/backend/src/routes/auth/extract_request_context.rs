//! Extracts request context data for analytics event tracking.
//!
//! Builds RequestData structure from HttpRequest for logging login events
//! and other analytics. Used by magic link verification endpoints.

/// Extract request context for magic link verification event tracking.
///
/// # Arguments
///
/// * `http_req` - The HTTP request to extract context from
/// * `_session_manager` - Session manager (unused but kept for signature compatibility)
///
/// # Returns
///
/// RequestData structure containing all request metadata for analytics
#[cfg(feature = "events")]
pub async fn extract_request_context_for_magic_link(
    http_req: &actix_web::HttpRequest,
    _session_manager: &std::sync::Arc<crate::services::session_manager::HybridSessionManager>,
) -> crate::services::events_service::request_context::RequestData {
    let method = http_req.method().to_string();
    let path = http_req.path().to_string();
    let query_string = http_req.query_string().to_string();
    let scheme = if http_req.connection_info().scheme() == "https" { "https" } else { "http" };
    let host = http_req.connection_info().host().to_string();
    let full_url = format!("{}://{}{}", scheme, host, path);
    
    let mut headers = std::collections::HashMap::new();
    for (name, value) in http_req.headers() {
        if let std::result::Result::Ok(value_str) = value.to_str() {
            headers.insert(name.to_string(), value_str.to_string());
        }
    }
    
    let connection_info = http_req.connection_info();
    let ip_address = connection_info.realip_remote_addr()
        .or_else(|| connection_info.peer_addr())
        .map(|addr| addr.split(':').next().unwrap_or(addr).to_string());
    
    let user_agent = headers.get("user-agent").cloned();
    let session_id = std::option::Option::None;
    
    crate::services::events_service::request_context::RequestData {
        method,
        path,
        full_url,
        query_string,
        headers,
        query_params: serde_json::json!({}),
        user_agent,
        ip_address,
        real_ip: std::option::Option::None,
        forwarded_for: std::option::Option::None,
        scheme: scheme.to_string(),
        host,
        port: std::option::Option::None,
        http_version: format!("{:?}", http_req.version()),
        content_type: std::option::Option::None,
        content_length: std::option::Option::None,
        content_encoding: std::option::Option::None,
        accept_language: std::option::Option::None,
        accept_encoding: std::option::Option::None,
        request_body: std::option::Option::None,
        request_body_size: std::option::Option::None,
        request_body_truncated: false,
        user_registration_date: std::option::Option::None,
        cookies: std::collections::HashMap::new(),
        request_id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now(),
        user_id: std::option::Option::None,
        session_id,
    }
}

/// No-op version when events feature is disabled.
#[cfg(not(feature = "events"))]
pub async fn extract_request_context_for_magic_link(
    _http_req: &actix_web::HttpRequest,
    _session_manager: &std::sync::Arc<crate::services::session_manager::HybridSessionManager>,
) -> () {
    ()
}

