//! Converts DNG images stored in GCS to web-compatible formats.
//!
//! This function provides DNG-specific conversion using ImageMagick with libraw support.
//! It downloads DNG files from GCS, converts them to WebP (default) or PNG,
//! uploads the result back to GCS, and cleans up the original file.

/// Converts a DNG image stored in GCS to a web-compatible format.
///
/// This function performs the following steps:
/// 0. Verifies ImageMagick is installed and supports DNG format
/// 1. Downloads the DNG file from GCS to a temporary location
/// 2. Uses ImageMagick to convert the DNG file to the specified format (WebP by default)
/// 3. Uploads the converted image file back to GCS
/// 4. Deletes the original DNG file from GCS (default behavior)
/// 5. Returns information about the converted file
///
/// # Arguments
/// * `gcs_client` - The GCS client for downloading/uploading files
/// * `bucket_name` - The name of the GCS bucket
/// * `dng_object_name` - The object name of the DNG file in GCS
/// * `output_format` - The desired output format (WebP by default)
///
/// # Returns
/// A `Result` containing `ConversionResult` with details about the converted file, or an error.
pub async fn convert_dng_on_gcs(
    gcs_client: &crate::services::gcs::gcs_client::GCSClient,
    bucket_name: &str,
    dng_object_name: &str,
    output_format: std::option::Option<crate::services::photo_extraction::output_format::OutputFormat>,
) -> anyhow::Result<crate::services::photo_extraction::conversion_result::ConversionResult> {
    crate::services::photo_extraction::convert_raw_image_on_gcs::convert_raw_image_on_gcs(
        gcs_client,
        bucket_name,
        dng_object_name,
        "DNG",
        &["dng", "DNG"],
        output_format,
        crate::services::photo_extraction::check_imagemagick_format_support::check_imagemagick_dng_support(),
    ).await
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[ignore] // This test requires local DNG file and ImageMagick with libraw support
    async fn test_dng_conversion_with_real_file() {
        let input_file = "/Users/pawelgodula/Downloads/L1004220.DNG";
        let output_dir = "/Users/pawelgodula/Downloads";
        
        // Skip test if input file doesn't exist
        if !std::path::Path::new(input_file).exists() {
            println!("⚠️  Skipping test - input DNG file not found: {}", input_file);
            return;
        }

        // First, verify ImageMagick can handle this DNG file
        match crate::services::photo_extraction::check_imagemagick_format_support::check_imagemagick_dng_support().await {
            std::result::Result::Ok(()) => println!("✅ ImageMagick DNG support confirmed"),
            std::result::Result::Err(e) => {
                println!("❌ ImageMagick DNG support check failed: {}", e);
                panic!("DNG support is required for this test");
            }
        }

        // Create temporary directory for processing
        let temp_id = uuid::Uuid::new_v4();
        let temp_dir = std::path::Path::new("/tmp").join(std::format!("dng_test_{}", temp_id));
        tokio::fs::create_dir_all(&temp_dir).await.expect("Failed to create temp directory");

        // Copy input file to temp directory
        let temp_dng_path = temp_dir.join("input.dng");
        tokio::fs::copy(input_file, &temp_dng_path).await.expect("Failed to copy DNG file");

        // Test WebP conversion (default)
        let format = crate::services::photo_extraction::output_format::OutputFormat::WebP;
        let output_path = temp_dir.join(std::format!("converted.{}", format.extension()));

        // Test ImageMagick conversion with same logic as production
        let mut command = tokio::process::Command::new("magick");
        command
            .arg("convert")
            .arg(&temp_dng_path)
            .arg("-quality")
            .arg("80")
            .arg("-define")
            .arg("webp:lossless=false")
            .arg(&output_path);

        println!("🔄 Converting DNG to WebP...");
        let start_time = std::time::Instant::now();
        let output = command.output().await.expect("Failed to execute ImageMagick");
        let duration = start_time.elapsed();

        if !output.status.success() {
            let stderr = std::string::String::from_utf8_lossy(&output.stderr);
            let stdout = std::string::String::from_utf8_lossy(&output.stdout);
            panic!(
                "ImageMagick conversion failed:\nSTDOUT: {}\nSTDERR: {}",
                stdout, stderr
            );
        }

        // Verify output file was created
        assert!(output_path.exists(), "WebP file should have been created");

        // Check file size
        let output_metadata = tokio::fs::metadata(&output_path).await.expect("Failed to read WebP metadata");
        assert!(output_metadata.len() > 0, "WebP file should not be empty");

        // Copy result to Downloads directory for manual inspection
        let final_output_path = std::path::Path::new(output_dir).join("L1004220_converted_test.webp");
        if let std::result::Result::Err(e) = tokio::fs::copy(&output_path, &final_output_path).await {
            println!("⚠️  Failed to copy result to Downloads: {}", e);
        } else {
            println!("📁 SAVED FOR INSPECTION: {}", final_output_path.display());
        }

        // Get original file size for comparison
        let original_metadata = tokio::fs::metadata(input_file).await.expect("Failed to read original DNG metadata");
        let compression_ratio = (output_metadata.len() as f64 / original_metadata.len() as f64) * 100.0;

        println!("✅ DNG conversion test successful!");
        println!("   Duration: {:.2}s", duration.as_secs_f64());
        println!("   Original DNG: {:.2} MB", original_metadata.len() as f64 / 1_048_576.0);
        println!("   WebP output: {:.2} MB", output_metadata.len() as f64 / 1_048_576.0);
        println!("   Compression: {:.1}% of original size", compression_ratio);

        // Clean up temp directory
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;
    }
} 