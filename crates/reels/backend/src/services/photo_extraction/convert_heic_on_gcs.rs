//! Converts HEIC images stored in GCS to web-compatible formats.
//!
//! This function provides HEIC-specific conversion using ImageMagick with libheif support.
//! It downloads HEIC files from GCS, converts them to WebP (default) or PNG,
//! uploads the result back to GCS, and cleans up the original file.

/// Converts a HEIC image stored in GCS to a web-compatible format.
///
/// This is a convenience wrapper around the generic RAW conversion pipeline
/// specifically configured for HEIC files with libheif support validation.
///
/// # Arguments
/// * `gcs_client` - The GCS client for downloading/uploading files
/// * `bucket_name` - The name of the GCS bucket
/// * `heic_object_name` - The object name of the HEIC file in GCS
/// * `output_format` - The desired output format (WebP by default)
///
/// # Returns
/// A `Result` containing `ConversionResult` with details about the converted file, or an error.
pub async fn convert_heic_on_gcs(
    gcs_client: &crate::services::gcs::gcs_client::GCSClient,
    bucket_name: &str,
    heic_object_name: &str,
    output_format: std::option::Option<crate::services::photo_extraction::output_format::OutputFormat>,
) -> anyhow::Result<crate::services::photo_extraction::conversion_result::ConversionResult> {
    crate::services::photo_extraction::convert_raw_image_on_gcs::convert_raw_image_on_gcs(
        gcs_client,
        bucket_name,
        heic_object_name,
        "HEIC",
        &["heic", "HEIC"],
        output_format,
        crate::services::photo_extraction::check_imagemagick_format_support::check_imagemagick_heic_support(),
    ).await
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[ignore] // This test requires ImageMagick with libheif support and real GCS access
    async fn test_convert_heic_to_webp() {
        // This test would require actual GCS setup and HEIC file
        // Skipped in normal test runs due to external dependencies
        println!("HEIC conversion test skipped - requires GCS and ImageMagick setup");
    }
} 