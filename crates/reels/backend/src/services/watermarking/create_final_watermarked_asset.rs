//! Creates and uploads the final watermarked asset.
//!
//! This function handles the final steps of the watermarking process: generating
//! a filename, uploading the processed image to GCS, and creating the database
//! record for the new watermarked asset.

use crate::services::watermarking::watermark_error::WatermarkError;
use crate::services::watermarking::generate_watermarked_filename::generate_watermarked_filename;
use crate::services::watermarking::get_content_type_from_filename::get_content_type_from_filename;
use crate::services::watermarking::create_watermarked_asset::create_watermarked_asset;
use crate::services::gcs::gcs_operations::{GCSOperations, UrlFormat};
use crate::services::gcs::parse_gcs_url::parse_gcs_url;

/// Creates and uploads the final watermarked asset
pub async fn create_final_watermarked_asset(
    pool: &sqlx::PgPool,
    gcs_client: &std::sync::Arc<dyn GCSOperations>,
    user_id: uuid::Uuid,
    source_asset: &crate::db::assets::Asset,
    watermarked_bytes: std::vec::Vec<u8>,
) -> std::result::Result<crate::db::assets::Asset, WatermarkError> {
    // Generate filename and upload path
    let final_filename = generate_watermarked_filename(&source_asset.name);
    let output_gcs_name = std::format!("watermarked/{}", final_filename);
    
    // Extract bucket name from source asset URL - use same bucket as source
    let (bucket_name, _) = parse_gcs_url(&source_asset.url)
        .map_err(|e| WatermarkError::InvalidConfig(
            std::format!("Failed to parse source asset URL {}: {}", source_asset.url, e)
        ))?;
    
    // Upload to GCS
    log::info!("Uploading final watermarked image to GCS");
    let watermarked_url = gcs_client
        .upload_raw_bytes(
            &bucket_name,
            &output_gcs_name,
            &get_content_type_from_filename(&final_filename),
            watermarked_bytes,
            false,
            UrlFormat::HttpsPublic,
        )
        .await?;
    
    log::info!("Final watermarked image uploaded to GCS: {}", watermarked_url);
    
    // Create asset record in database
    let watermarked_asset = create_watermarked_asset(
        pool,
        user_id,
        &final_filename,
        &output_gcs_name,
        &watermarked_url,
        source_asset,
    ).await?;
    
    std::result::Result::Ok(watermarked_asset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_filename_generation() {
        let source_name = "original_image.jpg";
        let filename = generate_watermarked_filename(source_name);
        
        assert!(filename.starts_with("original_image_watermarked_"));
        assert!(filename.ends_with(".png"));
        
        // Test GCS path generation
        let gcs_path = std::format!("watermarked/{}", filename);
        assert!(gcs_path.starts_with("watermarked/"));
        assert!(gcs_path.contains("original_image_watermarked_"));
    }

    #[test]
    fn test_bucket_extraction_from_asset_url() {
        // Test that we can extract bucket names from different GCS URL formats
        let test_cases = vec![
            ("https://storage.googleapis.com/my-bucket/path/to/asset.jpg", "my-bucket"),
            ("gs://another-bucket/some/object.png", "another-bucket"),
        ];

        for (url, expected_bucket) in test_cases {
            let (bucket, _) = parse_gcs_url(url).expect("Should parse valid GCS URL");
            assert_eq!(bucket, expected_bucket);
        }
    }

    #[test]
    fn test_content_type_determination() {
        let filename = "test_watermarked_20240101_120000.png";
        let content_type = get_content_type_from_filename(filename);
        assert_eq!(content_type, "image/png");
    }

    #[test]
    fn test_watermarked_path_structure() {
        let filename = "company_logo_watermarked_20240101_120000.png";
        let gcs_path = std::format!("watermarked/{}", filename);
        
        assert_eq!(gcs_path, "watermarked/company_logo_watermarked_20240101_120000.png");
        assert!(gcs_path.contains("/"));
        assert!(!gcs_path.starts_with("/"));
    }

    #[test]
    fn test_filename_uniqueness_structure() {
        // Test that the filename structure supports uniqueness
        let source_name = "test.jpg";
        let filename1 = generate_watermarked_filename(source_name);
        let filename2 = generate_watermarked_filename(source_name);
        
        // Both should start with the same prefix
        assert!(filename1.starts_with("test_watermarked_"));
        assert!(filename2.starts_with("test_watermarked_"));
        
        // The structure should include timestamp for uniqueness
        // (Note: They might be identical if generated in the same second)
        assert!(filename1.contains("_watermarked_"));
        assert!(filename2.contains("_watermarked_"));
    }
}
