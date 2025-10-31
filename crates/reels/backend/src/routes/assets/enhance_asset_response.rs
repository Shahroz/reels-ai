//! Response body for enhancing one or more existing assets with AI.
//!
//! Defines batched `EnhanceAssetResponse` with per-asset results and totals.

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct EnhanceAssetResponse {
    /// The original assets that were enhanced
    pub original_assets: std::vec::Vec<crate::db::assets::Asset>,
    /// Array of newly created enhanced assets derived from the original
    pub enhanced_assets: std::vec::Vec<crate::db::assets::Asset>,
    /// Total number of enhanced assets created for this original
    #[schema(example = 1)]
    pub total_enhanced: usize,
}