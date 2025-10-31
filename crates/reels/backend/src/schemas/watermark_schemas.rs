//! schemas for watermarking operations.
//!
//! This module defines request and response data structures for the watermarking API,
//! including detailed request schemas for applying different types of watermarks
//! and response schemas for watermarking jobs and results.
//! Supports both synchronous and asynchronous watermarking workflows.

/// Request schema for applying a watermark to an asset
#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct ApplyWatermarkRequest {
    #[schema(format = "uuid", value_type = String)]
    pub source_asset_id: uuid::Uuid,
    #[schema(format = "uuid", value_type = String)]
    pub logo_asset_id: uuid::Uuid,
    pub config: WatermarkConfig,
}

/// Request schema for applying multiple watermarks to an asset in a single operation
#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct ApplyBatchWatermarkRequest {
    #[schema(format = "uuid", value_type = String)]
    pub source_asset_id: uuid::Uuid,
    pub watermarks: std::vec::Vec<WatermarkDefinition>,
}

/// Definition of a single watermark in a batch operation
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct WatermarkDefinition {
    #[schema(format = "uuid", value_type = String)]
    pub logo_asset_id: uuid::Uuid,
    pub config: WatermarkConfig,
}

/// Watermark configuration schema
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct WatermarkConfig {
    pub position: WatermarkPosition,
    pub size: WatermarkSize,
    #[schema(minimum = 0.0, maximum = 1.0)]
    pub opacity: f32,
}

/// Watermark position configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(tag = "type", content = "value")]
#[schema(example = json!({"type": "custom", "value": {"x_percent": 50.0, "y_percent": 50.0}}))]
pub enum WatermarkPosition {
    #[serde(rename = "corner")]
    Corner(CornerPosition),
    #[serde(rename = "edge")]
    Edge(EdgePosition),
    #[serde(rename = "center")]
    Center,
    #[serde(rename = "custom")]
    Custom { x_percent: f32, y_percent: f32 },
}

/// Corner position options
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub enum CornerPosition {
    #[serde(rename = "top_left")]
    TopLeft,
    #[serde(rename = "top_right")]
    TopRight,
    #[serde(rename = "bottom_left")]
    BottomLeft,
    #[serde(rename = "bottom_right")]
    BottomRight,
}

/// Edge position options
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub enum EdgePosition {
    #[serde(rename = "top")]
    Top,
    #[serde(rename = "bottom")]
    Bottom,
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "right")]
    Right,
}

/// Watermark size configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(tag = "type", content = "value")]
#[schema(example = json!({"type": "percentage", "value": 15.0}))]
pub enum WatermarkSize {
    #[serde(rename = "percentage")]
    Percentage(f32),
    #[serde(rename = "absolute")]
    Absolute { width: u32, height: u32 },
    #[serde(rename = "fit_width")]
    FitWidth(u32),
    #[serde(rename = "fit_height")]
    FitHeight(u32),
}

/// Response schema for synchronous watermarking operations
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct WatermarkResponse {
    #[schema(format = "uuid", value_type = String)]
    pub result_asset_id: uuid::Uuid,
    pub result_asset_url: std::string::String,
    pub processing_time_ms: i64,
    #[schema(format = "date-time", value_type = String)]
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

/// Response schema for asynchronous watermarking jobs
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct WatermarkJobResponse {
    #[schema(format = "uuid", value_type = String)]
    pub job_id: uuid::Uuid,
    pub status: std::string::String,
    #[schema(format = "uuid", value_type = String, nullable = true)]
    pub result_asset_id: std::option::Option<uuid::Uuid>,
    pub result_asset_url: std::option::Option<std::string::String>,
    pub error_message: std::option::Option<std::string::String>,
    pub processing_time_ms: std::option::Option<i32>,
    #[schema(format = "date-time", value_type = String)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(format = "date-time", value_type = String)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Default for WatermarkConfig {
    fn default() -> Self {
        Self {
            position: WatermarkPosition::Corner(CornerPosition::BottomRight),
            size: WatermarkSize::Percentage(10.0),
            opacity: 0.8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watermark_config_validation_boundaries() {
        // Test default config is valid
        let config = WatermarkConfig::default();
        assert_eq!(config.opacity, 0.8);
        assert!(matches!(config.position, WatermarkPosition::Corner(CornerPosition::BottomRight)));
        assert!(matches!(config.size, WatermarkSize::Percentage(10.0)));
    }

    #[test]
    fn test_watermark_position_variants() {
        // Test all position variants can be created and serialized
        let positions = vec![
            WatermarkPosition::Corner(CornerPosition::TopLeft),
            WatermarkPosition::Corner(CornerPosition::TopRight),
            WatermarkPosition::Corner(CornerPosition::BottomLeft),
            WatermarkPosition::Corner(CornerPosition::BottomRight),
            WatermarkPosition::Edge(EdgePosition::Top),
            WatermarkPosition::Edge(EdgePosition::Bottom),
            WatermarkPosition::Edge(EdgePosition::Left),
            WatermarkPosition::Edge(EdgePosition::Right),
            WatermarkPosition::Center,
            WatermarkPosition::Custom { x_percent: 50.0, y_percent: 50.0 },
        ];

        for position in positions {
            let serialized = serde_json::to_string(&position).unwrap();
            let _deserialized: WatermarkPosition = serde_json::from_str(&serialized).unwrap();
        }
    }

    #[test]
    fn test_watermark_size_variants() {
        // Test all size variants can be created and serialized
        let sizes = vec![
            WatermarkSize::Percentage(15.0),
            WatermarkSize::Absolute { width: 200, height: 100 },
            WatermarkSize::FitWidth(300),
            WatermarkSize::FitHeight(150),
        ];

        for size in sizes {
            let serialized = serde_json::to_string(&size).unwrap();
            let _deserialized: WatermarkSize = serde_json::from_str(&serialized).unwrap();
        }
    }

    #[test]
    fn test_apply_watermark_request_edge_cases() {
        // Test with minimum opacity
        let json = r#"{
            "source_asset_id": "550e8400-e29b-41d4-a716-446655440000",
            "logo_asset_id": "550e8400-e29b-41d4-a716-446655440001",
            "config": {
                "position": {"type": "center"},
                "size": {"type": "percentage", "value": 1.0},
                "opacity": 0.0
            }
        }"#;

        let request: ApplyWatermarkRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.config.opacity, 0.0);
        assert!(matches!(request.config.position, WatermarkPosition::Center));
        assert!(matches!(request.config.size, WatermarkSize::Percentage(1.0)));

        // Test with maximum opacity
        let json = r#"{
            "source_asset_id": "550e8400-e29b-41d4-a716-446655440000",
            "logo_asset_id": "550e8400-e29b-41d4-a716-446655440001",
            "config": {
                "position": {"type": "custom", "value": {"x_percent": 0.0, "y_percent": 100.0}},
                "size": {"type": "absolute", "value": {"width": 1, "height": 1}},
                "opacity": 1.0
            }
        }"#;

        let request: ApplyWatermarkRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.config.opacity, 1.0);
        if let WatermarkPosition::Custom { x_percent, y_percent } = request.config.position {
            assert_eq!(x_percent, 0.0);
            assert_eq!(y_percent, 100.0);
        } else {
            panic!("Expected Custom position");
        }
    }
}
