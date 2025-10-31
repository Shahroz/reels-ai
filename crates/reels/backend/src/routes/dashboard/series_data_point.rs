//! Defines the structure for a single data point within a chart series.
//!
//! This module provides the `SeriesDataPoint` struct, which encapsulates
//! a single data value. The value itself is represented by `serde_json::Value`
//! to accommodate heterogeneous data types such as numbers, strings, or
//! custom JSON objects, as commonly found in chart data.
//! This approach provides a basic level of structure while retaining flexibility.

//! Revision History
//! - 2025-05-20T13:11:30Z @AI: Initial creation of SeriesDataPoint for structured chart series data.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct SeriesDataPoint {
    #[schema(
        value_type = utoipa::openapi::Object,
    )]
    pub value: serde_json::Value,
}