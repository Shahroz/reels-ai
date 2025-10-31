//! Analytics events database model with comprehensive middleware tracking.
//!
//! This module defines enhanced AnalyticsEvent structs for detailed cohort funnel analysis.
//! The middleware tracking captures comprehensive request/response data automatically,
//! while maintaining support for future custom event integration.
//! 
//! Event source and category are stored as strings in the database for flexibility,
//! with type safety enforced in Rust through enums and validation.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use std::net::IpAddr;

/// Event source type enum for Rust type safety
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum EventSource {
    Middleware,
    Custom,
}

impl EventSource {
    /// Convert to string for database storage
    pub fn as_str(&self) -> &'static str {
        match self {
            EventSource::Middleware => "middleware",
            EventSource::Custom => "custom",
        }
    }
    
    /// Parse from string with validation
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "middleware" => Ok(EventSource::Middleware),
            "custom" => Ok(EventSource::Custom),
            _ => Err(format!("Invalid event source: {}", s)),
        }
    }
}

/// Custom event category enum for Rust type safety
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum CustomEventCategory {
    Analytics,
    Engagement,
    Conversion,
    FeatureUsage,
    Other,
}

impl CustomEventCategory {
    /// Convert to string for database storage
    pub fn as_str(&self) -> &'static str {
        match self {
            CustomEventCategory::Analytics => "analytics",
            CustomEventCategory::Engagement => "engagement",
            CustomEventCategory::Conversion => "conversion",
            CustomEventCategory::FeatureUsage => "feature_usage",
            CustomEventCategory::Other => "other",
        }
    }
    
    /// Parse from string with validation
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "analytics" => Ok(CustomEventCategory::Analytics),
            "engagement" => Ok(CustomEventCategory::Engagement),
            "conversion" => Ok(CustomEventCategory::Conversion),
            "feature_usage" => Ok(CustomEventCategory::FeatureUsage),
            "other" => Ok(CustomEventCategory::Other),
            _ => Err(format!("Invalid custom event category: {}", s)),
        }
    }
}

/// Comprehensive user agent information parsed from raw user agent string
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserAgentDetails {
    pub raw_string: String,
    pub browser_family: String,
    pub browser_version: Option<String>,
    pub os_family: String,
    pub os_version: Option<String>,
    pub device_family: String,
    pub device_brand: Option<String>,
    pub device_model: Option<String>,
    pub is_mobile: bool,
    pub is_tablet: bool,
    pub is_pc: bool,
    pub is_bot: bool,
    pub is_touch_capable: bool,
    pub screen_resolution: Option<String>,
    pub viewport_size: Option<String>,
}

/// Request headers and metadata for comprehensive tracking
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RequestHeaders {
    pub accept: Option<String>,
    pub accept_encoding: Option<String>,
    pub accept_language: Option<String>,
    pub authorization: Option<String>, // Masked for security
    pub cache_control: Option<String>,
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
    pub host: Option<String>,
    pub origin: Option<String>,
    pub referer: Option<String>,
    pub user_agent: Option<String>,
    pub x_forwarded_for: Option<String>,
    pub x_real_ip: Option<String>,
    pub connection: Option<String>,
    pub upgrade_insecure_requests: Option<String>,
    pub sec_fetch_site: Option<String>,
    pub sec_fetch_mode: Option<String>,
    pub sec_fetch_dest: Option<String>,
}

/// Response metadata for comprehensive tracking
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResponseDetails {
    pub status_code: u16,
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
    pub cache_control: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub expires: Option<String>,
    pub location: Option<String>, // For redirects
    pub set_cookie: Option<Vec<String>>,
    pub x_frame_options: Option<String>,
    pub x_content_type_options: Option<String>,
    pub x_xss_protection: Option<String>,
    pub strict_transport_security: Option<String>,
}

/// Query parameters and form data for comprehensive request tracking
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RequestData {
    pub query_params: serde_json::Value,
    pub form_data: Option<serde_json::Value>,
    pub path_params: Option<serde_json::Value>,
    pub request_body_size: Option<u64>,
    pub request_body_hash: Option<String>, // SHA256 hash for deduplication
}

/// Performance and timing metrics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PerformanceMetrics {
    pub request_start_time: DateTime<Utc>,
    pub request_end_time: DateTime<Utc>,
    pub total_latency_ms: u128,
    pub processing_time_ms: u128,
    pub database_query_time_ms: Option<u128>,
    pub external_api_time_ms: Option<u128>,
    pub memory_usage_mb: Option<f64>,
    pub cpu_usage_percent: Option<f64>,
}

/// Geolocation and network information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NetworkDetails {
    pub ip_address: Option<IpAddr>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub timezone: Option<String>,
    pub isp: Option<String>,
    pub organization: Option<String>,
    pub connection_type: Option<String>, // mobile, broadband, etc.
    pub proxy_detected: bool,
    pub vpn_detected: bool,
}

/// Session and user context information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SessionContext {
    pub session_id: Option<String>,
    pub user_id: Option<Uuid>,
    pub user_registration_date: Option<NaiveDate>,
    pub user_authentication_status: String, // authenticated, anonymous, expired
    pub user_role: Option<String>,
    pub user_permissions: Option<Vec<String>>,
    pub session_duration_seconds: Option<u64>,
    pub session_page_views: Option<u32>,
    pub session_start_time: Option<DateTime<Utc>>,
    pub last_activity_time: Option<DateTime<Utc>>,
}

/// Error and exception details for comprehensive tracking
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorDetails {
    pub error_type: String,
    pub error_message: String,
    pub error_code: Option<String>,
    pub stack_trace: Option<String>,
    pub error_context: serde_json::Value,
    pub recovery_action: Option<String>,
}

/// Comprehensive middleware event details for automatic API request tracking
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MiddlewareEventDetails {
    // Core request information
    pub method: String,
    pub path: String,
    pub full_url: String,
    pub protocol: String, // HTTP/1.1, HTTP/2, etc.
    
    // Request details
    pub headers: RequestHeaders,
    pub request_data: RequestData,
    
    // Response details
    pub response: ResponseDetails,
    
    // Performance metrics
    pub performance: PerformanceMetrics,
    
    // Network and geolocation
    pub network: NetworkDetails,
    
    // User agent and device information
    pub user_agent: Option<UserAgentDetails>,
    
    // Session and user context
    pub session: SessionContext,
    
    // Error and exception tracking
    pub error_details: Option<ErrorDetails>,
    
    // Additional metadata
    pub metadata: serde_json::Value,
}

/// Custom event details for business-specific event tracking (future use)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CustomEventDetails {
    pub category: Option<CustomEventCategory>,
    pub tags: Option<Vec<String>>,
    pub properties: serde_json::Value,
    pub business_context: Option<serde_json::Value>,
}

/// Main analytics event database model
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AnalyticsEvent {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type = String)]
    pub id: Uuid,
    #[schema(example = "GET /api/dashboard")]
    pub event_name: String,
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001", format = "uuid", value_type = String)]
    pub user_id: Option<Uuid>,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub timestamp: DateTime<Utc>,
    /// Event source as string (stored in DB) - use source_enum() for type-safe access
    pub source: String,
    /// Event details stored as JSONB - contains comprehensive MiddlewareEventDetails or CustomEventDetails
    pub details: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<IpAddr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// User registration date for cohort-based funnel analysis
    pub user_registration_date: Option<NaiveDate>,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub created_at: DateTime<Utc>,
}

impl AnalyticsEvent {
    /// Get the event source as a type-safe enum
    pub fn source_enum(&self) -> Result<EventSource, String> {
        EventSource::from_str(&self.source)
    }
    
    /// Parse middleware event details from JSONB
    pub fn middleware_details(&self) -> Result<MiddlewareEventDetails, serde_json::Error> {
        serde_json::from_value(self.details.clone())
    }
    
    /// Parse custom event details from JSONB
    pub fn custom_details(&self) -> Result<CustomEventDetails, serde_json::Error> {
        serde_json::from_value(self.details.clone())
    }
}

/// New analytics event insert structure
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewAnalyticsEvent {
    pub event_name: String,
    pub user_id: Option<Uuid>,
    pub source: EventSource, // Use enum for type safety during creation
    pub details: serde_json::Value,
    pub session_id: Option<String>,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub user_registration_date: Option<NaiveDate>,
}

impl NewAnalyticsEvent {
    /// Convert to database-compatible format
    pub fn to_db_format(&self) -> (String, serde_json::Value) {
        (self.source.as_str().to_string(), self.details.clone())
    }
}

/// Cohort selection criteria for funnel analysis
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CohortCriteria {
    pub registration_date_start: NaiveDate,
    pub registration_date_end: NaiveDate,
    pub user_filters: Option<serde_json::Value>, // Additional user filtering criteria
}

/// Funnel analysis result structure
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FunnelAnalysisResult {
    pub cohort_criteria: CohortCriteria,
    pub total_cohort_users: u32,
    pub funnel_steps: Vec<FunnelStep>,
    pub conversion_rates: Vec<ConversionRate>,
    pub drop_off_analysis: Vec<DropOffPoint>,
    pub time_series_data: Vec<TimeSeriesData>,
    pub analysis_period: AnalysisPeriod,
}

/// Individual funnel step analysis
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FunnelStep {
    pub step_name: String,
    pub step_order: u32,
    pub unique_users: u32,
    pub total_events: u32,
    pub average_events_per_user: f64,
    pub median_time_to_step: Option<u64>, // seconds
    pub step_details: serde_json::Value,
}

/// Conversion rate between funnel steps
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConversionRate {
    pub from_step: String,
    pub to_step: String,
    pub conversion_rate: f64,
    pub conversion_count: u32,
    pub drop_off_count: u32,
}

/// Drop-off analysis for funnel optimization
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DropOffPoint {
    pub step_name: String,
    pub drop_off_count: u32,
    pub drop_off_percentage: f64,
    pub common_exit_paths: Vec<String>,
    pub time_to_drop_off: Option<u64>, // seconds
}

/// Time series data for trend analysis
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TimeSeriesData {
    pub date: NaiveDate,
    pub step_name: String,
    pub unique_users: u32,
    pub total_events: u32,
    pub conversion_rate: f64,
}

/// Analysis period definition
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AnalysisPeriod {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub analysis_type: String, // "daily", "weekly", "monthly"
}
