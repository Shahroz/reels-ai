//! Extracts frames from a video stored in GCS using ffmpeg and a signed URL.
//!
//! This module provides functionality to extract specific frames from a video file
//! located in Google Cloud Storage without downloading the entire file. It works by
//! generating a temporary V4 signed URL for the GCS object, which grants time-limited
//! read access. This URL is then passed to the `ffmpeg` command-line tool, which
//! streams the video content and extracts frames at the specified timestamps.

use anyhow::{anyhow, Context};
use google_cloud_storage::{
    client::{Client, ClientConfig},
    sign::{SignedURLMethod, SignedURLOptions},
};
use std::path::Path;
use tokio::process::Command;

// Add import for GCS client

/// Extracts specific frames from a video on GCS and saves them locally.
///
/// This function performs the following steps:
/// 1. Creates a GCS client using Application Default Credentials.
/// 2. Generates a V4 signed URL for the given GCS object, valid for 15 minutes.
/// 3. For each provided timestamp, it spawns an `ffmpeg` process.
/// 4. `ffmpeg` uses the signed URL to stream the video and extract the frame.
/// 5. The extracted frame is saved to a temporary directory.
/// 6. The function returns a list of paths to the saved image files.
///
/// # Arguments
/// * `gcs_uri` - The GS URI of the video file (e.g., "gs://bucket-name/video.mp4").
/// * `timestamps` - A slice of strings representing the timestamps to extract (e.g., "00:00:03").
///
/// # Returns
/// A `Result` containing a `Vec<String>` of file paths for the extracted frames, or an error.
pub async fn extract_frames_from_video_on_gcs(
    gcs_uri: &str,
    timestamps: &[&str],
) -> anyhow::Result<Vec<String>> {
    // 1. Create GCS Client
    let config = ClientConfig::default().with_auth().await.context("Failed to create GCS client config")?;
    let client = Client::new(config);

    // Parse the GCS URI to get bucket and object name
    let (bucket_name, object_name) = parse_gcs_uri(gcs_uri)?;

    // 2. Generate Signed URL
    let options = SignedURLOptions {
        method: SignedURLMethod::GET,
        expires: std::time::Duration::from_secs(15 * 60), // 15 minutes
        ..Default::default()
    };
    let signed_url = client
        .signed_url(&bucket_name, &object_name, None, None, options)
        .await
        .context("Failed to create signed URL")?;

    // 3. Define the output directory and ensure it exists
    let output_dir = Path::new("crates/narrativ/backend/tests/_output");
    std::fs::create_dir_all(output_dir).context("Failed to create output directory")?;

    // Get the base name from the object name for the filename
    let base_name = Path::new(&object_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("video_frame");

    let mut output_paths = Vec::new();

    // 4. Loop through timestamps and extract frames
    for (index, timestamp) in timestamps.iter().enumerate() {
        let safe_timestamp = timestamp.replace(":", "_");
        let image_filename = format!("{}_{}_frame_{}.png", base_name, safe_timestamp, index + 1);
        let output_path = output_dir.join(image_filename);

        let mut command = Command::new("ffmpeg");
        command
            .arg("-i")
            .arg(&signed_url)
            .arg("-ss")
            .arg(timestamp)
            .arg("-frames:v")
            .arg("1")
            .arg("-y") // Overwrite output file if it exists
            .arg(&output_path);

        let output = command.output().await.context("Failed to execute ffmpeg command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!(
                "ffmpeg failed for timestamp {}: {}\nSTDOUT: {}\nSTDERR: {}",
                timestamp,
                output.status,
                String::from_utf8_lossy(&output.stdout),
                stderr
            ));
        }

        output_paths.push(output_path.to_str().unwrap().to_string());
    }

    Ok(output_paths)
}

/// Parses a "gs://bucket/object" URI into its components.
fn parse_gcs_uri(uri: &str) -> anyhow::Result<(String, String)> {
    let stripped = uri
        .strip_prefix("gs://")
        .ok_or_else(|| anyhow!("Invalid GCS URI format: must start with gs://"))?;
    let mut parts = stripped.splitn(2, '/');
    let bucket = parts
        .next()
        .ok_or_else(|| anyhow!("Invalid GCS URI: missing bucket name"))?
        .to_string();
    let object = parts
        .next()
        .ok_or_else(|| anyhow!("Invalid GCS URI: missing object name"))?
        .to_string();
    Ok((bucket, object))
}

#[cfg(test)]
mod tests {
    use crate::services::gcs::gcs_client::GCSClient;
use crate::services::gcs::gcs_operations::UrlFormat;
    use super::*;

    #[tokio::test]
    #[ignore] // This test uses real GCP resources and requires `ffmpeg` to be installed.
    async fn test_extract_frames_from_gcs_video() {
        // Ensure .env file is loaded for GOOGLE_APPLICATION_CREDENTIALS
        dotenvy::dotenv().ok();

        let gcs_uri = "gs://real-estate-videos/output_10s.mp4";
        let timestamps = ["00:00:02", "00:00:05", "00:00:08"];

        let result = extract_frames_from_video_on_gcs(gcs_uri, &timestamps).await;

        assert!(result.is_ok(), "Function failed: {:?}", result.err());

        let paths = result.unwrap();
        assert_eq!(paths.len(), 3, "Expected 3 frames to be extracted");

        for path in &paths {
            println!("Extracted frame to: {}", path);
            assert!(
                std::fs::metadata(path).is_ok(),
                "Output file not found: {}",
                path
            );
            // You could add more assertions here, e.g., checking file size > 0
        }
    }

    #[tokio::test]
    #[ignore] // This test uses real GCP resources and requires `ffmpeg` to be installed.
    async fn test_compression_impact_on_image_quality() {
        // Ensure .env file is loaded for GOOGLE_APPLICATION_CREDENTIALS
        dotenvy::dotenv().ok();

        let gcs_uri = "gs://real-estate-videos/output_10s.mp4";
        let timestamp = "00:00:00";
        let bucket_name = "real-estate-videos";

        // Create GCS client for uploading results
        let gcs_client = GCSClient::new();

        // Create signed URL for the video
        let config = ClientConfig::default().with_auth().await.context("Failed to create GCS client config").unwrap();
        let client = Client::new(config);
        let (bucket, object) = parse_gcs_uri(gcs_uri).unwrap();

        let options = SignedURLOptions {
            method: SignedURLMethod::GET,
            expires: std::time::Duration::from_secs(15 * 60), // 15 minutes
            ..Default::default()
        };
        let signed_url = client
            .signed_url(&bucket, &object, None, None, options)
            .await
            .context("Failed to create signed URL")
            .unwrap();

        // Define output directory and ensure it exists
        let output_dir = Path::new("crates/narrativ/backend/tests/_output");
        std::fs::create_dir_all(output_dir).context("Failed to create output directory").unwrap();

        // Test different compression formats
        let mut results = Vec::new();

        // 1. PNG (no compression)
        let png_path = output_dir.join("TEST_IMG_QUALITY_frame_png.png");
        let mut command = Command::new("ffmpeg");
        command
            .arg("-i")
            .arg(&signed_url)
            .arg("-ss")
            .arg(timestamp)
            .arg("-frames:v")
            .arg("1")
            .arg("-y") // Overwrite output file if it exists
            .arg(&png_path);

        let output = command.output().await.context("Failed to execute ffmpeg command for PNG").unwrap();
        assert!(output.status.success(), "ffmpeg failed for PNG: {}", String::from_utf8_lossy(&output.stderr));

        let png_bytes = std::fs::read(&png_path).context("Failed to read PNG file").unwrap();
        let png_size = png_bytes.len();
        let png_url = gcs_client
            .upload_raw_bytes(
                bucket_name,
                "TEST_IMG_QUALITY_frame.png",
                "image/png",
                png_bytes,
                false,
                UrlFormat::HttpsPublic,
            )
            .await
            .context("Failed to upload PNG to GCS")
            .unwrap();
        results.push(("PNG (no compression)", png_size, png_url));

        // 2. JPEG with high quality (low compression)
        let jpeg_high_path = output_dir.join("TEST_IMG_QUALITY_frame_jpeg_high.jpg");
        let mut command = Command::new("ffmpeg");
        command
            .arg("-i")
            .arg(&signed_url)
            .arg("-ss")
            .arg(timestamp)
            .arg("-frames:v")
            .arg("1")
            .arg("-q:v")
            .arg("2") // High quality (low compression)
            .arg("-y")
            .arg(&jpeg_high_path);

        let output = command.output().await.context("Failed to execute ffmpeg command for JPEG high quality").unwrap();
        assert!(output.status.success(), "ffmpeg failed for JPEG high quality: {}", String::from_utf8_lossy(&output.stderr));

        let jpeg_high_bytes = std::fs::read(&jpeg_high_path).context("Failed to read JPEG high quality file").unwrap();
        let jpeg_high_size = jpeg_high_bytes.len();
        let jpeg_high_url = gcs_client
            .upload_raw_bytes(
                bucket_name,
                "TEST_IMG_QUALITY_frame_jpeg_high.jpg",
                "image/jpeg",
                jpeg_high_bytes,
                false,
                UrlFormat::HttpsPublic,
            )
            .await
            .context("Failed to upload JPEG high quality to GCS")
            .unwrap();
        results.push(("JPEG (high quality)", jpeg_high_size, jpeg_high_url));

        // 3. JPEG with low quality (high compression)
        let jpeg_low_path = output_dir.join("TEST_IMG_QUALITY_frame_jpeg_low.jpg");
        let mut command = Command::new("ffmpeg");
        command
            .arg("-i")
            .arg(&signed_url)
            .arg("-ss")
            .arg(timestamp)
            .arg("-frames:v")
            .arg("1")
            .arg("-q:v")
            .arg("31") // Low quality (high compression)
            .arg("-y")
            .arg(&jpeg_low_path);

        let output = command.output().await.context("Failed to execute ffmpeg command for JPEG low quality").unwrap();
        assert!(output.status.success(), "ffmpeg failed for JPEG low quality: {}", String::from_utf8_lossy(&output.stderr));

        let jpeg_low_bytes = std::fs::read(&jpeg_low_path).context("Failed to read JPEG low quality file").unwrap();
        let jpeg_low_size = jpeg_low_bytes.len();
        let jpeg_low_url = gcs_client
            .upload_raw_bytes(
                bucket_name,
                "TEST_IMG_QUALITY_frame_jpeg_low.jpg",
                "image/jpeg",
                jpeg_low_bytes,
                false,
                UrlFormat::HttpsPublic,
            )
            .await
            .context("Failed to upload JPEG low quality to GCS")
            .unwrap();
        results.push(("JPEG (low quality)", jpeg_low_size, jpeg_low_url));

        // 4. WebP
        let webp_path = output_dir.join("TEST_IMG_QUALITY_frame_webp.webp");
        let mut command = Command::new("ffmpeg");
        command
            .arg("-i")
            .arg(&signed_url)
            .arg("-ss")
            .arg(timestamp)
            .arg("-frames:v")
            .arg("1")
            .arg("-c:v")
            .arg("libwebp")
            .arg("-quality")
            .arg("80") // WebP quality
            .arg("-y")
            .arg(&webp_path);

        let output = command.output().await.context("Failed to execute ffmpeg command for WebP").unwrap();
        assert!(output.status.success(), "ffmpeg failed for WebP: {}", String::from_utf8_lossy(&output.stderr));

        let webp_bytes = std::fs::read(&webp_path).context("Failed to read WebP file").unwrap();
        let webp_size = webp_bytes.len();
        let webp_url = gcs_client
            .upload_raw_bytes(
                bucket_name,
                "TEST_IMG_QUALITY_frame.webp",
                "image/webp",
                webp_bytes,
                false,
                UrlFormat::HttpsPublic,
            )
            .await
            .context("Failed to upload WebP to GCS")
            .unwrap();
        results.push(("WebP", webp_size, webp_url));

        // Print results
        println!("\n=== Compression Impact Analysis ===");
        println!("Video: {}", gcs_uri);
        println!("Timestamp: {}", timestamp);
        println!("Results uploaded to GCS bucket: {}", bucket_name);
        println!();

        for (format, size, url) in &results {
            println!("{}: {} bytes ({:.2} KB) - {}", format, size, *size as f64 / 1024.0, url);
        }

        // Calculate compression ratios relative to PNG
        let png_size_f64 = results[0].1 as f64;
        println!("\n=== Compression Ratios (relative to PNG) ===");
        for (format, size, _) in &results {
            let ratio = (*size as f64 / png_size_f64) * 100.0;
            println!("{}: {:.1}% of PNG size", format, ratio);
        }

        // Clean up local files
        let _ = std::fs::remove_file(&png_path);
        let _ = std::fs::remove_file(&jpeg_high_path);
        let _ = std::fs::remove_file(&jpeg_low_path);
        let _ = std::fs::remove_file(&webp_path);

        // Basic assertions
        assert!(!results.is_empty(), "No results were generated");
        assert!(results[0].1 > 0, "PNG file size should be greater than 0");
        assert!(results[1].1 > 0, "JPEG high quality file size should be greater than 0");
        assert!(results[2].1 > 0, "JPEG low quality file size should be greater than 0");
        assert!(results[3].1 > 0, "WebP file size should be greater than 0");

        // Compression effectiveness assertions
        assert!(results[2].1 < results[1].1, "JPEG low quality should be smaller than JPEG high quality");
        assert!(results[1].1 < results[0].1, "JPEG high quality should be smaller than PNG");
    }

    #[test]
    fn test_parse_gcs_uri_success() {
        let (bucket, object) = parse_gcs_uri("gs://my-bucket-123/path/to/my/video.mp4").unwrap();
        assert_eq!(bucket, "my-bucket-123");
        assert_eq!(object, "path/to/my/video.mp4");
    }

    #[test]
    fn test_parse_gcs_uri_failures() {
        assert!(parse_gcs_uri("http://google.com").is_err());
        assert!(parse_gcs_uri("gs://just-a-bucket").is_err());
        assert!(parse_gcs_uri("gs://").is_err());
    }
} 