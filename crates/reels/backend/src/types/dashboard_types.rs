//! Types for dashboard queries

use serde::{Deserialize, Serialize};

/// Entity types for activity tracking
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActivityEntityType {
    Users,
    Documents,
    Styles,
    Assets,
    CustomCreativeFormats,
    Creatives,
}

/// Date ranges for KPI metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllDateRanges {
    pub today_start: chrono::DateTime<chrono::Utc>,
    pub today_end: chrono::DateTime<chrono::Utc>,
    pub yesterday_start: chrono::DateTime<chrono::Utc>,
    pub yesterday_end: chrono::DateTime<chrono::Utc>,
    pub this_week_start: chrono::DateTime<chrono::Utc>,
    pub this_week_end: chrono::DateTime<chrono::Utc>,
    pub last_week_start: chrono::DateTime<chrono::Utc>,
    pub last_week_end: chrono::DateTime<chrono::Utc>,
    pub this_month_start: chrono::DateTime<chrono::Utc>,
    pub this_month_end: chrono::DateTime<chrono::Utc>,
    pub last_month_start: chrono::DateTime<chrono::Utc>,
    pub last_month_end: chrono::DateTime<chrono::Utc>,
    pub last_7_days_start: chrono::DateTime<chrono::Utc>,
    pub last_7_days_end: chrono::DateTime<chrono::Utc>,
    pub prev_7_days_start: chrono::DateTime<chrono::Utc>,
    pub prev_7_days_end: chrono::DateTime<chrono::Utc>,
    pub last_30_days_start: chrono::DateTime<chrono::Utc>,
    pub last_30_days_end: chrono::DateTime<chrono::Utc>,
    pub prev_30_days_start: chrono::DateTime<chrono::Utc>,
    pub prev_30_days_end: chrono::DateTime<chrono::Utc>,
}

