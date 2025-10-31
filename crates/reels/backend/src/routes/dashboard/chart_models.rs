//! Defines data structures for representing chart data.
//!
//! This module contains structs used to construct and serialize data
//! suitable for rendering charts in the dashboard. It includes models
//! for individual data series and the overall chart structure,
//! accommodating various data types (numbers, strings, or structured objects)
//! for chart points via `serde_json::Value`.
//!
//! Revision History
//! - 2025-05-20T13:11:30Z @AI: Modified ChartSeries to use structured SeriesDataPoint.
//! - 2025-05-20T12:58:44Z @AI: Initial creation of chart data models.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ChartSeries {
    #[schema(example = "Sales")]
    pub name: String,
    #[schema(
        value_type = Vec<crate::routes::dashboard::series_data_point::SeriesDataPoint>,
    )]
    pub data: std::vec::Vec<crate::routes::dashboard::series_data_point::SeriesDataPoint>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ChartData {
    #[schema(example = "Monthly Performance")]
    pub title: Option<String>,
    pub labels: Vec<String>,
    pub series: Vec<ChartSeries>,
}