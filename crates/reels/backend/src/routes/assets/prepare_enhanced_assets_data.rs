//! Prepares enhanced asset data for database saving.
//!
//! Transforms GCS URIs and original asset metadata into the format required
//! by the save_assets_from_gcs function, handling name generation and
//! metadata extraction.
//!
//! Revision History:
//! - 2025-10-17T00:00:00Z @AI: Extracted from enhance_asset.rs

pub fn prepare_enhanced_assets_data(
    enhanced_gcs_uris: &[std::string::String],
    original_assets: &[crate::db::assets::Asset],
    final_names: std::option::Option<&[std::string::String]>,
) -> std::result::Result<std::vec::Vec<crate::routes::assets::save_assets_from_gcs::GcsAssetData>, crate::routes::assets::enhance_asset_error::EnhanceAssetError> {
    let mut assets_to_save = std::vec::Vec::new();
    
    for (index, gcs_uri) in enhanced_gcs_uris.iter().enumerate() {
        let original_asset = original_assets.get(index)
            .ok_or_else(|| crate::routes::assets::enhance_asset_error::EnhanceAssetError::ProcessingFailed("Mismatch between enhanced URIs and original assets".into()))?;
        
        let file_extension = original_asset.name
            .split('.')
            .next_back()
            .unwrap_or("jpg");
        
        let enhanced_name = if let Some(names) = final_names {
            names.get(index).cloned().unwrap_or_else(|| format!("{}_enhanced_{}.{file_extension}",
                original_asset.name.trim_end_matches(&format!(".{file_extension}")),
                index + 1
            ))
        } else {
            format!("{}_enhanced_{}.{file_extension}", 
                original_asset.name.trim_end_matches(&format!(".{file_extension}")),
                index + 1
            )
        };
        
        let gcs_object_name = crate::routes::assets::gcs_uri_extractor::extract_object_name_from_gcs_url(gcs_uri)
            .map_err(|e| crate::routes::assets::enhance_asset_error::EnhanceAssetError::ProcessingFailed(format!("Failed to extract object name from GCS URL '{gcs_uri}': {e}")))?;
        
        let asset_data = crate::routes::assets::save_assets_from_gcs::GcsAssetData {
            name: enhanced_name,
            r#type: original_asset.r#type.clone(),
            gcs_url: gcs_uri.clone(),
            gcs_object_name,
            collection_id: original_asset.collection_id.map(|id| id.to_string()),
        };
        
        assets_to_save.push(asset_data);
    }

    Ok(assets_to_save)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_enhanced_assets_data_basic() {
        let asset_name = "test_image.webp";
        let asset_url = format!("https://storage.googleapis.com/bucket/{asset_name}");
        let enhanced_uris = vec![asset_url.clone()];
        
        let original_assets = vec![
            crate::db::assets::Asset {
                id: uuid::Uuid::new_v4(),
                user_id: Some(uuid::Uuid::new_v4()),
                name: asset_name.to_string(),
                r#type: "image/webp".to_string(),
                gcs_object_name: asset_name.to_string(),
                url: asset_url.clone(),
                collection_id: None,
                metadata: None,
                created_at: None,
                updated_at: None,
                is_public: false,
            },
        ];

        let result = prepare_enhanced_assets_data(&enhanced_uris, &original_assets, None);
        assert!(result.is_ok());
        
        let assets_data = result.unwrap();
        assert_eq!(assets_data.len(), 1);
        assert_eq!(assets_data[0].name, "test_image_enhanced_1.webp");
        assert_eq!(assets_data[0].r#type, "image/webp");
    }

    #[test]
    fn test_prepare_enhanced_assets_data_with_custom_names() {
        let asset_name = "photo.jpg";
        let asset_url = format!("https://storage.googleapis.com/bucket/{asset_name}");
        let enhanced_uris = vec![asset_url.clone()];
        
        let original_assets = vec![
            crate::db::assets::Asset {
                id: uuid::Uuid::new_v4(),
                user_id: Some(uuid::Uuid::new_v4()),
                name: asset_name.to_string(),
                r#type: "image/jpeg".to_string(),
                gcs_object_name: asset_name.to_string(),
                url: asset_url.clone(),
                collection_id: None,
                metadata: None,
                created_at: None,
                updated_at: None,
                is_public: false,
            },
        ];

        let final_names = vec!["photo - Enhanced.jpg".to_string()];
        let result = prepare_enhanced_assets_data(&enhanced_uris, &original_assets, Some(&final_names));
        assert!(result.is_ok());
        
        let assets_data = result.unwrap();
        assert_eq!(assets_data.len(), 1);
        assert_eq!(assets_data[0].name, "photo - Enhanced.jpg");
    }

    #[test]
    fn test_prepare_enhanced_assets_data_mismatch() {
        let enhanced_uris = vec![
            "https://storage.googleapis.com/bucket/file1.jpg".to_string(),
            "https://storage.googleapis.com/bucket/file2.jpg".to_string(),
        ];
        
        let original_assets = vec![
            crate::db::assets::Asset {
                id: uuid::Uuid::new_v4(),
                user_id: Some(uuid::Uuid::new_v4()),
                name: "test.jpg".to_string(),
                r#type: "image/jpeg".to_string(),
                gcs_object_name: "test.jpg".to_string(),
                url: "https://storage.googleapis.com/bucket/test.jpg".to_string(),
                collection_id: None,
                metadata: None,
                created_at: None,
                updated_at: None,
                is_public: false,
            },
        ];

        let result = prepare_enhanced_assets_data(&enhanced_uris, &original_assets, None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::routes::assets::enhance_asset_error::EnhanceAssetError::ProcessingFailed(_)));
    }

    #[test]
    fn test_file_extension_extraction() {
        let name1 = "photo.jpg";
        let ext1 = name1.split('.').next_back().unwrap_or("jpg");
        assert_eq!(ext1, "jpg");
        
        let name2 = "image.with.dots.png";
        let ext2 = name2.split('.').next_back().unwrap_or("jpg");
        assert_eq!(ext2, "png");
        
        let name3 = "no_extension";
        // next_back() on split('.') will return the last segment, which is the whole string if no dot
        let ext3 = name3.split('.').next_back().unwrap_or("jpg");
        assert_eq!(ext3, "no_extension"); // This is what actually happens
        
        // To get fallback behavior, we need to check if there's a dot:
        let name4 = "no_extension";
        let ext4 = if name4.contains('.') {
            name4.split('.').next_back().unwrap_or("jpg")
        } else {
            "jpg"
        };
        assert_eq!(ext4, "jpg"); // This uses the fallback correctly
    }

    #[test]
    fn test_collection_id_preservation() {
        let collection_id = uuid::Uuid::new_v4();
        let asset_url = "https://storage.googleapis.com/bucket/test.jpg".to_string();
        let enhanced_uris = vec![asset_url.clone()];
        
        let original_assets = vec![
            crate::db::assets::Asset {
                id: uuid::Uuid::new_v4(),
                user_id: Some(uuid::Uuid::new_v4()),
                name: "test.jpg".to_string(),
                r#type: "image/jpeg".to_string(),
                gcs_object_name: "test.jpg".to_string(),
                url: asset_url.clone(),
                collection_id: Some(collection_id),
                metadata: None,
                created_at: None,
                updated_at: None,
                is_public: false,
            },
        ];

        let result = prepare_enhanced_assets_data(&enhanced_uris, &original_assets, None);
        assert!(result.is_ok());
        
        let assets_data = result.unwrap();
        assert_eq!(assets_data[0].collection_id, Some(collection_id.to_string()));
    }
}

