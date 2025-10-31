//! Common utilities for custom events tracking.
//!
//! This module provides shared functions for extracting metrics, building request context,
//! and calculating derived values for custom analytics events. These utilities ensure
//! consistency across all event types while avoiding duplication of logic.

/// Builds standard request details from request context data
/// This creates the `request_details` JSONB that's consistent across all events
/// EXACT same structure as vocal tour events for consistency
pub fn build_standard_request_details(
    request_context: &crate::services::events_service::request_context::RequestData,
) -> serde_json::Value {
    let user_agent = request_context.user_agent.as_deref().unwrap_or("Unknown");
    let session_type = request_context.session_id.as_ref()
        .map(|s| if s.starts_with("jwt_") { "jwt" } else { "cookie" })
        .unwrap_or("none");

    serde_json::json!({
        // Request metadata - FIRST in the structure
        "host": request_context.host,
        "path": request_context.path,
        "method": request_context.method,
        "scheme": request_context.scheme,
        "query_string": request_context.query_string,
        
        // Navigation context
        "origin": request_context.headers.get("origin").cloned(),
        "referer": request_context.headers.get("referer").cloned(),
        
        // Essential user context
        "ip_address": request_context.ip_address,
        "user_agent": request_context.user_agent,
        "browser_info": {
            "browser_family": extract_browser_family(user_agent),
            "browser_version": extract_browser_version(user_agent),
            "os_family": extract_os_family(user_agent),
            "is_mobile": is_mobile_device(user_agent),
            "is_desktop": is_desktop_device(user_agent)
        },
        
        // Authentication & session state
        "session_type": session_type,
        "is_authenticated": request_context.user_id.is_some()
    })
}

/// Extracts email domain from email address
pub fn extract_email_domain(email: &str) -> String {
    email.split('@')
        .nth(1)
        .unwrap_or("unknown")
        .to_lowercase()
}

/// Extracts UTM parameters from request headers if available
pub fn extract_utm_params(request_context: &crate::services::events_service::request_context::RequestData) -> serde_json::Value {
    // Extract UTM parameters from referer or query string if available
    // For now, return empty object - could be enhanced to parse actual UTM params
    serde_json::json!({})
}

/// Get user count statistics from database  
pub async fn get_user_count_stats(pool: &sqlx::PgPool, user_id: uuid::Uuid) -> Result<UserCountStats, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT 
            (SELECT COUNT(*) FROM assets WHERE user_id = $1) as total_assets,
            (SELECT COUNT(*) FROM vocal_tours WHERE user_id = $1) as total_vocal_tours,
            (SELECT COUNT(*) FROM documents WHERE user_id = $1) as total_documents,
            (SELECT COUNT(*) FROM collections WHERE user_id = $1) as total_collections
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(UserCountStats {
        total_assets: result.total_assets.unwrap_or(0),
        total_vocal_tours: result.total_vocal_tours.unwrap_or(0),
        total_documents: result.total_documents.unwrap_or(0),
        total_collections: result.total_collections.unwrap_or(0),
    })
}

/// Get user trial information from database
pub async fn get_user_trial_info(pool: &sqlx::PgPool, user_id: uuid::Uuid) -> Result<UserTrialInfo, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT 
            u.created_at,
            u.email,
            -- Note: subscription status would come from billing tables if available
            'trial' as subscription_status
        FROM users u 
        WHERE u.id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?;
    
    let days_since_registration = (chrono::Utc::now() - result.created_at).num_days();
    let trial_days_remaining = std::cmp::max(0, 14 - days_since_registration); // Assuming 14-day trial
    
    Ok(UserTrialInfo {
        days_since_registration: days_since_registration as i32,
        trial_days_remaining: trial_days_remaining as i32,
        subscription_status: result.subscription_status.unwrap_or_else(|| "trial".to_string()),
        email: result.email,
    })
}

/// Data structure for user count statistics
#[derive(Debug, Clone)]
pub struct UserCountStats {
    pub total_assets: i64,
    pub total_vocal_tours: i64,
    pub total_documents: i64,
    pub total_collections: i64,
}

/// Data structure for user trial information
#[derive(Debug, Clone)]
pub struct UserTrialInfo {
    pub days_since_registration: i32,
    pub trial_days_remaining: i32,
    pub subscription_status: String,
    pub email: String,
}

// Browser detection functions (simplified versions)
fn extract_browser_family(user_agent: &str) -> String {
    if user_agent.contains("Chrome") {
        "Chrome".to_string()
    } else if user_agent.contains("Firefox") {
        "Firefox".to_string() 
    } else if user_agent.contains("Safari") {
        "Safari".to_string()
    } else if user_agent.contains("Edge") {
        "Edge".to_string()
    } else {
        "Unknown".to_string()
    }
}

fn extract_browser_version(user_agent: &str) -> String {
    // Simplified version extraction - could be enhanced with regex
    if user_agent.contains("Chrome") {
        if let Some(start) = user_agent.find("Chrome/") {
            let version_start = start + 7;
            if let Some(end) = user_agent[version_start..].find(' ') {
                return user_agent[version_start..version_start + end].to_string();
            }
        }
    }
    "Unknown".to_string()
}

fn extract_os_family(user_agent: &str) -> String {
    if user_agent.contains("Windows") {
        "Windows".to_string()
    } else if user_agent.contains("Mac OS") || user_agent.contains("macOS") {
        "macOS".to_string()
    } else if user_agent.contains("Linux") {
        "Linux".to_string()
    } else if user_agent.contains("iOS") {
        "iOS".to_string()
    } else if user_agent.contains("Android") {
        "Android".to_string()
    } else {
        "Unknown".to_string()
    }
}

fn is_mobile_device(user_agent: &str) -> bool {
    user_agent.contains("Mobile") || 
    user_agent.contains("Android") || 
    user_agent.contains("iPhone") || 
    user_agent.contains("iPad")
}

fn is_desktop_device(user_agent: &str) -> bool {
    !is_mobile_device(user_agent)
} 
 
 