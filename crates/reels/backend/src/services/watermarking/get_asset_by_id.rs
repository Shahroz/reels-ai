//! Gets an asset by ID with user authorization.
//!
//! This function retrieves an asset from the database while ensuring it belongs to the specified user.
//! Provides proper error handling for missing assets and converts database rows to Asset structs.
//! Essential for maintaining data security and user isolation.

use crate::services::watermarking::watermark_error::WatermarkError;

/// Gets an asset by ID, ensuring it belongs to the user
pub async fn get_asset_by_id(
    pool: &sqlx::PgPool,
    asset_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> std::result::Result<crate::db::assets::Asset, WatermarkError> {
    let asset = sqlx::query!(
        "SELECT id, user_id, name, type, gcs_object_name, url, created_at, updated_at, collection_id, metadata FROM assets WHERE id = $1 AND user_id = $2",
        asset_id,
        user_id
    )
    .fetch_optional(pool)
    .await?;
    
    if let std::option::Option::Some(row) = asset {
        std::result::Result::Ok(crate::db::assets::Asset {
            id: row.id,
            user_id: row.user_id,
            name: row.name,
            r#type: row.r#type,
            gcs_object_name: row.gcs_object_name,
            url: row.url,
            collection_id: row.collection_id,
            metadata: row.metadata,
            created_at: std::option::Option::Some(row.created_at),
            updated_at: std::option::Option::Some(row.updated_at),
            is_public: false,
        })
    } else {
        std::result::Result::Err(WatermarkError::AssetNotFound(asset_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_not_found_error() {
        let asset_id = uuid::Uuid::new_v4();
        let error = WatermarkError::AssetNotFound(asset_id);
        
        // Test that the error type is correct and contains the asset ID for debugging
        assert!(matches!(error, WatermarkError::AssetNotFound(_)));
        let error_msg = error.to_string();
        assert!(error_msg.contains(&asset_id.to_string()));
    }

    #[test]
    fn test_asset_construction() {
        let asset_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        
        let asset = crate::db::assets::Asset {
            id: asset_id,
            user_id: std::option::Option::Some(user_id),
            name: "test.jpg".to_string(),
            r#type: "image/jpeg".to_string(),
            gcs_object_name: "test/test.jpg".to_string(),
            url: "https://example.com/test.jpg".to_string(),
            collection_id: std::option::Option::None,
            metadata: std::option::Option::None,
            created_at: std::option::Option::Some(chrono::Utc::now()),
            updated_at: std::option::Option::Some(chrono::Utc::now()),
            is_public: false,
        };
        
        assert_eq!(asset.id, asset_id);
        assert_eq!(asset.user_id, std::option::Option::Some(user_id));
        assert_eq!(asset.name, "test.jpg");
    }
}
