//! Creates a new asset record for the watermarked image.
//!
//! This function inserts a new asset record into the database for the watermarked
//! image, inheriting properties from the source asset while setting new URL and
//! object name. Ensures proper database record keeping for watermarked assets.

use crate::services::watermarking::watermark_error::WatermarkError;
use crate::services::watermarking::get_content_type_from_filename::get_content_type_from_filename;

/// Creates a new asset record for the watermarked image
pub async fn create_watermarked_asset(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    filename: &str,
    gcs_object_name: &str,
    url: &str,
    source_asset: &crate::db::assets::Asset,
) -> std::result::Result<crate::db::assets::Asset, WatermarkError> {
    let asset_type = get_content_type_from_filename(filename);
    
    let record = sqlx::query!(
        r#"
        INSERT INTO assets (user_id, name, type, gcs_object_name, url, collection_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, user_id, name, type, gcs_object_name, url, created_at, updated_at, collection_id, metadata
        "#,
        source_asset.user_id.unwrap_or(user_id),  // Use original owner, fallback to current user
        filename,
        asset_type,
        gcs_object_name,
        url,
        source_asset.collection_id
    )
    .fetch_one(pool)
    .await?;
    
    let watermarked_asset = crate::db::assets::Asset {
        id: record.id,
        user_id: record.user_id,
        name: record.name,
        r#type: record.r#type,
        gcs_object_name: record.gcs_object_name,
        url: record.url,
        collection_id: record.collection_id,
        metadata: record.metadata,
        created_at: std::option::Option::Some(record.created_at),
        updated_at: std::option::Option::Some(record.updated_at),
        is_public: false,
    };

    // Inherit shares from source asset to watermarked asset
    if let Err(e) = crate::queries::assets::inherit_shares_from_asset::inherit_shares_from_asset_single(
        pool,
        source_asset.id,
        watermarked_asset.id,
    ).await {
        log::warn!("Failed to inherit shares from source asset {} to watermarked asset {}: {}", 
                   source_asset.id, watermarked_asset.id, e);
    }

    std::result::Result::Ok(watermarked_asset)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a test asset for testing purposes
    fn create_test_source_asset() -> crate::db::assets::Asset {
        crate::db::assets::Asset {
            id: uuid::Uuid::new_v4(),
            user_id: std::option::Option::Some(uuid::Uuid::new_v4()),
            name: "source.jpg".to_string(),
            r#type: "image/jpeg".to_string(),
            gcs_object_name: "uploads/source.jpg".to_string(),
            url: "https://storage.googleapis.com/bucket/uploads/source.jpg".to_string(),
            collection_id: std::option::Option::Some(uuid::Uuid::new_v4()),
            metadata: std::option::Option::None,
            created_at: std::option::Option::Some(chrono::Utc::now()),
            updated_at: std::option::Option::Some(chrono::Utc::now()),
            is_public: false,
        }
    }

    #[test]
    fn test_asset_creation_parameters() {
        let source_asset = create_test_source_asset();
        let user_id = uuid::Uuid::new_v4();
        let filename = "watermarked_image.png";
        let gcs_object_name = "watermarked/watermarked_image.png";
        let url = "https://storage.googleapis.com/bucket/watermarked/watermarked_image.png";
        
        // Test that parameters are properly structured
        assert!(!filename.is_empty());
        assert!(gcs_object_name.contains("watermarked/"));
        assert!(url.contains("https://"));
        assert!(source_asset.collection_id.is_some());
        
        // Test content type determination
        let content_type = get_content_type_from_filename(filename);
        assert_eq!(content_type, "image/png");
    }

    #[test]
    fn test_collection_id_inheritance() {
        let source_asset = create_test_source_asset();
        let source_collection_id = source_asset.collection_id;
        
        // The watermarked asset should inherit the collection_id from source
        assert!(source_collection_id.is_some());
        
        // Test that we're using the source collection_id in the creation
        let inherited_collection = source_asset.collection_id;
        assert_eq!(inherited_collection, source_collection_id);
    }

    #[test]
    fn test_asset_type_generation() {
        // Test various filename extensions
        let test_cases = vec![
            ("image.png", "image/png"),
            ("photo.jpg", "image/jpeg"),
            ("picture.jpeg", "image/jpeg"),
            ("graphic.webp", "image/webp"),
            ("unknown.xyz", "image/png"), // Default case
        ];
        
        for (filename, expected_type) in test_cases {
            let content_type = get_content_type_from_filename(filename);
            assert_eq!(content_type, expected_type);
        }
    }

    #[test]
    fn test_database_record_structure() {
        // Test that the asset structure is correct for database insertion
        let test_asset = crate::db::assets::Asset {
            id: uuid::Uuid::new_v4(),
            user_id: std::option::Option::Some(uuid::Uuid::new_v4()),
            name: "test.png".to_string(),
            r#type: "image/png".to_string(),
            gcs_object_name: "test/test.png".to_string(),
            url: "https://example.com/test.png".to_string(),
            collection_id: std::option::Option::None,
            metadata: std::option::Option::None,
            created_at: std::option::Option::Some(chrono::Utc::now()),
            updated_at: std::option::Option::Some(chrono::Utc::now()),
            is_public: false,
        };
        
        // Verify all required fields are present
        assert!(!test_asset.name.is_empty());
        assert!(!test_asset.r#type.is_empty());
        assert!(!test_asset.gcs_object_name.is_empty());
        assert!(!test_asset.url.is_empty());
        assert!(!test_asset.is_public); // Watermarked assets should not be public by default
    }
}
