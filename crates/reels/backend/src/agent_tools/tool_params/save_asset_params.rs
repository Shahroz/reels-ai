//! Defines parameters for the `save_asset` agent tool.
//!
//! This structure encapsulates the parameters for saving multiple assets
//! with existing GCS URIs to the assets table. The tool skips
//! the upload step and directly saves the asset metadata.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AssetData {
    #[schemars(description = "Name of the asset (e.g., 'my-image.png')")]
    pub name: String,
    #[schemars(description = "MIME type of the asset (e.g., 'image/png', 'application/pdf')")]
    pub r#type: String,
    #[schemars(description = "Full GCS URL of the existing asset")]
    pub gcs_url: String,
    #[schemars(description = "GCS object name/path (e.g., 'user-id/asset-id.png')")]
    pub gcs_object_name: String,
    #[schemars(description = "Optional collection ID to associate the asset with")]
    pub collection_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, Default)]
pub struct SaveAssetParams {
    #[schemars(description = "Array of assets to save. Each asset requires name, type, GCS URL, and object name.")]
    pub assets: Vec<AssetData>,
    #[schemars(skip)]
    pub user_id: Option<uuid::Uuid>,
} 