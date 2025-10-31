//! Generic RAW image conversion function for GCS-stored files.
//!
//! This core function handles the conversion of any RAW image format to web-compatible
//! formats using ImageMagick. It's used by both HEIC and DNG specific conversion functions
//! and provides a unified pipeline for downloading, converting, and uploading images.

use anyhow::Context;

/// RAII cleanup helper for temporary directories
struct TemporaryDirectoryCleanup {
    path: std::path::PathBuf,
}

impl std::ops::Drop for TemporaryDirectoryCleanup {
    fn drop(&mut self) {
        if let std::result::Result::Err(e) = std::fs::remove_dir_all(&self.path) {
            log::warn!("Failed to cleanup temporary directory {:?}: {}", self.path, e);
        }
    }
}

/// Converts a RAW image stored in GCS to a web-compatible format.
///
/// This function performs the following steps:
/// 0. Verifies ImageMagick is installed and supports the source format
/// 1. Downloads the RAW file from GCS to a temporary location
/// 2. Uses ImageMagick to convert the RAW file to the specified format (WebP by default)
/// 3. Uploads the converted image file back to GCS
/// 4. Deletes the original RAW file from GCS (default behavior)
/// 5. Returns information about the converted file
///
/// # Arguments
/// * `gcs_client` - The GCS client for downloading/uploading files
/// * `bucket_name` - The name of the GCS bucket
/// * `raw_object_name` - The object name of the RAW file in GCS
/// * `source_format` - The source format name (e.g., "HEIC", "DNG")
/// * `source_extensions` - Array of possible file extensions for replacement
/// * `output_format` - The desired output format (WebP by default)
/// * `format_check_fn` - Function to check if ImageMagick supports the source format
///
/// # Returns
/// A `Result` containing `ConversionResult` with details about the converted file, or an error.
pub async fn convert_raw_image_on_gcs(
    gcs_client: &crate::services::gcs::gcs_client::GCSClient,
    bucket_name: &str,
    raw_object_name: &str,
    source_format: &str,
    source_extensions: &[&str],
    output_format: std::option::Option<crate::services::photo_extraction::output_format::OutputFormat>,
    format_check_fn: impl std::future::Future<Output = anyhow::Result<()>>,
) -> anyhow::Result<crate::services::photo_extraction::conversion_result::ConversionResult> {
    // 0. First check if ImageMagick is available and supports the source format
    format_check_fn.await
        .context("ImageMagick environment check failed")?;
    
    // 1. Determine output format (WebP by default)
    let format = output_format.unwrap_or_default();
    
    // 2. Create temporary directory for processing
    let temp_id = uuid::Uuid::new_v4();
    let temp_dir = std::path::Path::new("/tmp").join(std::format!("raw_conversion_{}", temp_id));
    std::fs::create_dir_all(&temp_dir).context("Failed to create temporary directory")?;

    // Ensure cleanup happens even if function fails
    let _cleanup = TemporaryDirectoryCleanup { path: temp_dir.clone() };

    // 3. Download RAW file from GCS
    let fallback_filename = std::format!("input.{}", source_extensions[0]);
    let raw_filename = std::path::Path::new(raw_object_name)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(&fallback_filename);
    let local_raw_path = temp_dir.join(raw_filename);

    let download_error_msg = std::format!("Failed to download {} file from GCS", source_format);
    let raw_bytes = gcs_client
        .download_object_as_bytes(bucket_name, raw_object_name)
        .await
        .context(download_error_msg)?;

    let write_error_msg = std::format!("Failed to write {} file to temporary location", source_format);
    tokio::fs::write(&local_raw_path, raw_bytes)
        .await
        .context(write_error_msg)?;

    // 4. Generate output filename based on format
    let base_name = std::path::Path::new(raw_object_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("converted");
    
    // Replace all possible source extensions with the target extension
    let mut output_object_name = raw_object_name.to_string();
    for ext in source_extensions {
        output_object_name = output_object_name
            .replace(&std::format!(".{}", ext), &std::format!(".{}", format.extension()))
            .replace(&std::format!(".{}", ext.to_uppercase()), &std::format!(".{}", format.extension()));
    }
    
    let local_output_path = temp_dir.join(std::format!("{}.{}", base_name, format.extension()));

    // 5. Convert RAW to target format using ImageMagick
    let mut command = tokio::process::Command::new("magick");
    command
        .arg("convert")
        .arg(&local_raw_path);
    
    // Add format-specific options
    match format {
        crate::services::photo_extraction::output_format::OutputFormat::WebP => {
            command
                .arg("-quality")
                .arg("80") // Good balance of quality vs file size for WebP
                .arg("-define")
                .arg("webp:lossless=false");
        }
        crate::services::photo_extraction::output_format::OutputFormat::Png => {
            // PNG doesn't need special quality settings
        }
    }
    
    command.arg(&local_output_path);

    let output = command.output().await.context("Failed to execute ImageMagick convert command")?;

    if !output.status.success() {
        let stderr = std::string::String::from_utf8_lossy(&output.stderr);
        return std::result::Result::Err(anyhow::anyhow!(
            "ImageMagick conversion failed: {}\nSTDOUT: {}\nSTDERR: {}",
            output.status,
            std::string::String::from_utf8_lossy(&output.stdout),
            stderr
        ));
    }

    // 6. Verify output file was created and has content
    if !local_output_path.exists() {
        return std::result::Result::Err(anyhow::anyhow!("{} file was not created by ImageMagick", format.extension().to_uppercase()));
    }

    let output_metadata = tokio::fs::metadata(&local_output_path)
        .await
        .context("Failed to read output file metadata")?;

    if output_metadata.len() == 0 {
        return std::result::Result::Err(anyhow::anyhow!("Generated {} file is empty", format.extension().to_uppercase()));
    }

    // 7. Upload converted file to GCS
    let output_bytes = tokio::fs::read(&local_output_path)
        .await
        .context("Failed to read converted output file")?;

    let output_url = gcs_client
        .upload_raw_bytes(
            bucket_name,
            &output_object_name,
            format.content_type(),
            output_bytes,
            false, // not public by default - will be made public via standard URL patterns
            crate::services::gcs::gcs_operations::UrlFormat::HttpsPublic,
        )
        .await
        .context("Failed to upload converted file to GCS")?;

    // 8. Delete original RAW file (default behavior)
    if let std::result::Result::Err(e) = gcs_client.delete_object(bucket_name, raw_object_name).await {
        log::warn!("Failed to delete original {} file {}: {}", source_format, raw_object_name, e);
        // Don't fail the entire operation if cleanup fails
    }

    log::info!(
        "Successfully converted {} to {}: {} -> {} ({} bytes)",
        source_format,
        format.extension().to_uppercase(),
        raw_object_name,
        output_object_name,
        output_metadata.len()
    );

    std::result::Result::Ok(crate::services::photo_extraction::conversion_result::ConversionResult::new(
        output_object_name,
        output_url,
        format.content_type().to_string(),
        format.extension().to_string(),
        format,
    ))
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[ignore] // This test requires ImageMagick and real GCS access
    async fn test_convert_raw_image_generic() {
        // This test would require actual GCS setup and RAW file
        // Skipped in normal test runs due to external dependencies
        println!("Generic RAW conversion test skipped - requires GCS and ImageMagick setup");
    }
} 