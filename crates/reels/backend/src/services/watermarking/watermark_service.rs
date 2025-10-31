//! Core watermarking service for applying logos to assets.
//!
//! This service handles the complete watermarking workflow including asset download,
//! image processing with ImageMagick, and new asset creation.
//! Supports both synchronous processing for small images and async jobs for larger ones.

use crate::schemas::watermark_schemas::{WatermarkConfig, WatermarkResponse};
use crate::services::watermarking::imagemagick_commands::{
    check_imagemagick_watermark_support, ImageMagickError
};
use crate::services::gcs::gcs_operations::{GCSOperations, UrlFormat};
use crate::services::gcs::parse_gcs_url::parse_gcs_url;

/// Error types for watermarking operations
#[derive(Debug, thiserror::Error)]
pub enum WatermarkError {
    #[error("Asset not found: {0}")]
    AssetNotFound(uuid::Uuid),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Image processing error: {0}")]
    ImageProcessing(std::string::String),
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),
    #[error("GCS error: {0}")]
    Gcs(#[from] anyhow::Error),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(std::string::String),
    #[error("ImageMagick command error: {0}")]
    ImageMagickCommand(#[from] ImageMagickError),
    #[error("File size limit exceeded: {0} bytes (max: {1} bytes)")]
    FileSizeExceeded(u64, u64),
    #[error("Processing timeout exceeded")]
    ProcessingTimeout,
}

/// Maximum file size for watermarking operations (2GB)
const MAX_FILE_SIZE_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/// Maximum processing timeout (5 minutes)
const PROCESSING_TIMEOUT_SECONDS: u64 = 5 * 60;

/// Synchronously applies a watermark to an asset
pub async fn apply_watermark_sync(
    pool: &sqlx::PgPool,
    gcs_client: &std::sync::Arc<dyn GCSOperations>,
    user_id: uuid::Uuid,
    source_asset_id: uuid::Uuid,
    logo_asset_id: uuid::Uuid,
    config: WatermarkConfig,
) -> std::result::Result<WatermarkResponse, WatermarkError> {
    log::info!("Starting watermark application - user: {}, source: {}, logo: {}", user_id, source_asset_id, logo_asset_id);
    log::info!("Watermark config: {:?}", config);
    let start_time = std::time::Instant::now();
    
    // Validate ImageMagick availability
    log::info!("Checking ImageMagick availability");
    check_imagemagick_watermark_support()
        .await
        .map_err(|e| {
            log::error!("ImageMagick support check failed: {}", e);
            WatermarkError::ImageProcessing(e.to_string())
        })?;
    log::info!("ImageMagick availability check passed");

    // Fetch source asset
    log::info!("Fetching source asset: {}", source_asset_id);
    let source_asset = get_asset_by_id(pool, source_asset_id, user_id).await.map_err(|e| {
        log::error!("Failed to fetch source asset {}: {}", source_asset_id, e);
        e
    })?;
    log::info!("Source asset fetched: {} ({})", source_asset.name, source_asset.url);
    
    // Fetch logo asset
    log::info!("Fetching logo asset: {}", logo_asset_id);
    let logo_asset = get_asset_by_id(pool, logo_asset_id, user_id).await.map_err(|e| {
        log::error!("Failed to fetch logo asset {}: {}", logo_asset_id, e);
        e
    })?;
    log::info!("Logo asset fetched: {} ({})", logo_asset.name, logo_asset.url);
    
    // Validate image types
    log::info!("Validating image asset types");
    validate_image_assets(&source_asset, &logo_asset).map_err(|e| {
        log::error!("Image asset validation failed: {}", e);
        e
    })?;
    log::info!("Image asset validation passed");
    
    // Create secure temporary directory for processing
    let temp_id = uuid::Uuid::new_v4();
    let temp_dir = std::env::temp_dir().join(std::format!("watermark_{}", temp_id));
    tokio::fs::create_dir_all(&temp_dir).await?;
    
    // Download source image
    let source_local_path = temp_dir.join("source_image");
    log::info!("Downloading source asset from: {}", source_asset.url);
    download_asset_to_file(gcs_client, &source_asset.url, &source_local_path).await?;
    validate_file_size(&source_local_path).await?;
    log::info!("Source image downloaded to: {:?}", source_local_path);
    
    // Download logo image
    let logo_local_path = temp_dir.join("logo_image");
    log::info!("Downloading logo asset from: {}", logo_asset.url);
    download_asset_to_file(gcs_client, &logo_asset.url, &logo_local_path).await?;
    validate_file_size(&logo_local_path).await?;
    log::info!("Logo image downloaded to: {:?}", logo_local_path);
    
    // Prepare resized logo path
    let resized_logo_path = temp_dir.join("logo_resized");
    
    // Step 1: Resize logo if needed
    log::info!("Resizing logo with config: {:?}", config.size);
    resize_logo_if_needed(&logo_local_path, &resized_logo_path, &config.size).await?;
    
    // Use resized logo for further processing
    let resized_logo_path_final = if resized_logo_path.exists() {
        log::info!("Using resized logo: {:?}", resized_logo_path);
        resized_logo_path
    } else {
        log::info!("Using original logo (no resize needed): {:?}", logo_local_path);
        logo_local_path.clone()
    };
    
    // Step 2: Apply opacity to logo separately (if needed)
    let opacity_processed_logo_path = temp_dir.join("logo_with_opacity");
    let final_logo_path = if config.opacity < 1.0 {
        log::info!("Applying opacity {} to logo separately", config.opacity);
        apply_opacity_to_logo(&resized_logo_path_final, &opacity_processed_logo_path, config.opacity).await?;
        log::info!("Logo opacity applied, using processed logo: {:?}", opacity_processed_logo_path);
        opacity_processed_logo_path
    } else {
        log::info!("No opacity adjustment needed, using resized logo");
        resized_logo_path_final
    };
    
    // Prepare output path
    let output_filename = generate_watermarked_filename(&source_asset.name);
    let output_local_path = temp_dir.join(&output_filename);
    
    // Step 3: Composite the pre-processed logo onto the original image
    log::info!("Building composite command for final composition");
    let command = build_brightness_preserving_composite_command(
        &source_local_path,
        &final_logo_path,
        &output_local_path,
        &config,
    )?;
    
    log::info!("Executing ImageMagick composite command: {:?}", command.args);
    execute_imagemagick_command_with_timeout(command).await?;
    log::info!("ImageMagick composite command completed successfully");
    
    // Verify output file was created
    if !output_local_path.exists() {
        return std::result::Result::Err(WatermarkError::ImageProcessing(
            std::string::String::from("Watermarked image was not created")
        ));
    }
    
    // Upload watermarked image to GCS
    let output_gcs_name = std::format!("watermarked/{}", output_filename);
    let watermarked_bytes = tokio::fs::read(&output_local_path).await?;
    
    // Get bucket name from environment
    let bucket_name = std::env::var("GCS_BUCKET").map_err(|_| WatermarkError::InvalidConfig(
        std::string::String::from("GCS_BUCKET environment variable not set")
    ))?;
    
    let watermarked_url = gcs_client
        .upload_raw_bytes(
            &bucket_name,
            &output_gcs_name,
            &get_content_type_from_filename(&output_filename),
            watermarked_bytes,
            false,
            UrlFormat::HttpsPublic,
        )
        .await?;
    
    // Create new asset record
    let watermarked_asset = create_watermarked_asset(
        pool,
        user_id,
        &output_filename,
        &output_gcs_name,
        &watermarked_url,
        &source_asset,
    ).await?;
    
    // Clean up temporary files with proper error logging
    if let std::result::Result::Err(cleanup_error) = tokio::fs::remove_dir_all(&temp_dir).await {
        tracing::warn!("Failed to clean up temporary directory {}: {}", temp_dir.display(), cleanup_error);
    }
    
    let processing_time_ms = start_time.elapsed().as_millis() as i64;
    
    std::result::Result::Ok(WatermarkResponse {
        result_asset_id: watermarked_asset.id,
        result_asset_url: watermarked_asset.url,
        processing_time_ms,
        completed_at: chrono::Utc::now(),
    })
}

/// Gets an asset by ID, ensuring it belongs to the user
async fn get_asset_by_id(
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
            created_at: std::option::Option::Some(row.created_at),
            updated_at: std::option::Option::Some(row.updated_at),
            collection_id: row.collection_id,
            metadata: row.metadata,
        })
    } else {
        std::result::Result::Err(WatermarkError::AssetNotFound(asset_id))
    }
}

/// Validates that assets are suitable for watermarking
fn validate_image_assets(
    source_asset: &crate::db::assets::Asset,
    logo_asset: &crate::db::assets::Asset,
) -> std::result::Result<(), WatermarkError> {
    // Check if source asset is an image (check for MIME type starting with "image/")
    if !source_asset.r#type.starts_with("image/") {
        return std::result::Result::Err(WatermarkError::InvalidConfig(
            std::format!("Source asset must be an image, got: {}", source_asset.r#type)
        ));
    }
    
    // Check if logo asset is an image (check for MIME type starting with "image/")
    if !logo_asset.r#type.starts_with("image/") {
        return std::result::Result::Err(WatermarkError::InvalidConfig(
            std::format!("Logo asset must be an image, got: {}", logo_asset.r#type)
        ));
    }
    
    log::info!("Asset validation passed - source: {}, logo: {}", source_asset.r#type, logo_asset.r#type);
    std::result::Result::Ok(())
}

/// Downloads an asset from GCS to a local file
async fn download_asset_to_file(
    gcs_client: &std::sync::Arc<dyn GCSOperations>,
    asset_url: &str,
    local_path: &std::path::Path,
) -> std::result::Result<(), WatermarkError> {
    log::info!("Parsing GCS URL: {}", asset_url);
    // Parse the GCS URL to get bucket and object name
    let (bucket_name, object_name) = parse_gcs_url(asset_url)
        .map_err(|e| {
            log::error!("Failed to parse GCS URL '{}': {}", asset_url, e);
            WatermarkError::InvalidConfig(std::format!("Invalid GCS URL: {}", e))
        })?;
    
    log::info!("Parsed GCS URL - bucket: '{}', object: '{}'", bucket_name, object_name);
    
    // Get the concrete GCS client for direct access to download_object_as_bytes
    let gcs_concrete_client = gcs_client.as_any().downcast_ref::<crate::services::gcs::gcs_client::GCSClient>()
        .ok_or_else(|| WatermarkError::InvalidConfig(std::string::String::from("Expected GCSClient")))?;
    
    log::info!("Downloading object from GCS bucket '{}', object '{}'", bucket_name, object_name);
    let bytes = gcs_concrete_client
        .download_object_as_bytes(&bucket_name, &object_name)
        .await
        .map_err(|e| {
            log::error!("Failed to download asset from GCS: {}", e);
            WatermarkError::Gcs(e)
        })?;
    
    log::info!("Downloaded {} bytes, writing to local file: {:?}", bytes.len(), local_path);
    tokio::fs::write(local_path, bytes).await?;
    log::info!("Successfully wrote asset to local file: {:?}", local_path);
    
    std::result::Result::Ok(())
}

/// Validates file size against maximum limit
async fn validate_file_size(file_path: &std::path::Path) -> std::result::Result<(), WatermarkError> {
    let metadata = tokio::fs::metadata(file_path).await?;
    let file_size = metadata.len();
    
    if file_size > MAX_FILE_SIZE_BYTES {
        return std::result::Result::Err(WatermarkError::FileSizeExceeded(file_size, MAX_FILE_SIZE_BYTES));
    }
    
    std::result::Result::Ok(())
}

/// Applies opacity to logo image separately to preserve brightness during compositing
async fn apply_opacity_to_logo(
    logo_path: &std::path::Path,
    output_path: &std::path::Path,
    opacity: f32,
) -> std::result::Result<(), WatermarkError> {
    log::info!("Applying opacity {} to logo from {:?} to {:?}", opacity, logo_path, output_path);
    
    // Use magick convert to modify the alpha channel
    let mut cmd = tokio::process::Command::new("magick");
    cmd.arg("convert")
       .arg(logo_path.to_string_lossy().to_string())
       .arg("-alpha")
       .arg("set")
       .arg("-channel")
       .arg("A")
       .arg("-evaluate")
       .arg("multiply")
       .arg(opacity.to_string())
       .arg("+channel")
       .arg(output_path.to_string_lossy().to_string());
    
    log::info!("Executing ImageMagick opacity command: magick convert {} -alpha set -channel A -evaluate multiply {} +channel {}", 
               logo_path.display(), opacity, output_path.display());
    
    let output = cmd.output().await?;
    
    if !output.status.success() {
        let stderr = std::string::String::from_utf8_lossy(&output.stderr);
        log::error!("ImageMagick opacity command failed with status: {}", output.status);
        log::error!("STDERR: {}", stderr);
        return std::result::Result::Err(WatermarkError::ImageProcessing(
            std::string::String::from("ImageMagick opacity command failed")
        ));
    }
    
    log::info!("Logo opacity application completed successfully");
    std::result::Result::Ok(())
}

/// Resizes logo if size configuration requires it
async fn resize_logo_if_needed(
    logo_path: &std::path::Path,
    resized_logo_path: &std::path::Path,
    size_config: &crate::schemas::watermark_schemas::WatermarkSize,
) -> std::result::Result<(), WatermarkError> {
    use crate::schemas::watermark_schemas::WatermarkSize;
    
    // Check if resizing is needed
    let needs_resize = match size_config {
        WatermarkSize::Percentage(_) => true,
        WatermarkSize::Absolute { .. } => true,
        WatermarkSize::FitWidth(_) => true,
        WatermarkSize::FitHeight(_) => true,
    };
    
    if !needs_resize {
        log::info!("No resize needed for size config: {:?}", size_config);
        return std::result::Result::Ok(());
    }
    
    log::info!("Resizing logo from {:?} to {:?} with config: {:?}", logo_path, resized_logo_path, size_config);
    
    // Build resize command
    let mut args = std::vec::Vec::new();
    args.push(std::string::String::from("convert"));
    args.push(logo_path.to_string_lossy().to_string());
    
    // Add resize arguments
    match size_config {
        WatermarkSize::Percentage(percent) => {
            args.push(std::string::String::from("-resize"));
            args.push(std::format!("{}%", percent));
        }
        WatermarkSize::Absolute { width, height } => {
            args.push(std::string::String::from("-resize"));
            args.push(std::format!("{}x{}", width, height));
        }
        WatermarkSize::FitWidth(width) => {
            args.push(std::string::String::from("-resize"));
            args.push(std::format!("{}x", width));
        }
        WatermarkSize::FitHeight(height) => {
            args.push(std::string::String::from("-resize"));
            args.push(std::format!("x{}", height));
        }
    }
    
    args.push(resized_logo_path.to_string_lossy().to_string());
    
    // Execute resize command
    log::info!("Executing ImageMagick resize command: magick {}", args.join(" "));
    let mut cmd = tokio::process::Command::new("magick");
    cmd.args(&args);
    
    let output = cmd.output().await?;
    
    if !output.status.success() {
        let stderr = std::string::String::from_utf8_lossy(&output.stderr);
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        log::error!("ImageMagick resize command failed with status: {}", output.status);
        log::error!("STDOUT: {}", stdout);
        log::error!("STDERR: {}", stderr);
        return std::result::Result::Err(WatermarkError::ImageProcessing(
            std::format!("Logo resize failed: {}\nSTDOUT: {}\nSTDERR: {}", output.status, stdout, stderr)
        ));
    }
    
    log::info!("Logo resize completed successfully");
    std::result::Result::Ok(())
}

/// Builds brightness-preserving composite command for pre-processed logos
fn build_brightness_preserving_composite_command(
    source_image_path: &std::path::Path,
    logo_path: &std::path::Path,
    output_path: &std::path::Path,
    config: &crate::schemas::watermark_schemas::WatermarkConfig,
) -> std::result::Result<crate::services::watermarking::imagemagick_commands::ImageMagickCommand, crate::services::watermarking::imagemagick_commands::ImageMagickError> {
    // Validate all input paths for security
    use crate::services::watermarking::imagemagick_commands::validate_path_security;
    validate_path_security(source_image_path)?;
    validate_path_security(logo_path)?;
    validate_path_security(output_path)?;
    
    let mut args = std::vec::Vec::new();
    
    // Use composite command syntax: composite [options] overlay base output
    // Since opacity is already applied to the logo, we use simple Over composition
    
    // Add positioning arguments using -gravity and -geometry
    add_modern_position_args(&mut args, &config.position);
    
    // Use Over composition without any opacity adjustments (logo is pre-processed)
    args.push(std::string::String::from("-compose"));
    args.push(std::string::String::from("Over"));
    
    // composite command syntax: overlay base output
    args.push(logo_path.to_string_lossy().to_string());
    args.push(source_image_path.to_string_lossy().to_string());
    args.push(output_path.to_string_lossy().to_string());
    
    // Final validation of all arguments
    use crate::services::watermarking::imagemagick_commands::validate_imagemagick_args;
    validate_imagemagick_args(&args)?;
    
    use crate::services::watermarking::imagemagick_commands::ImageMagickCommand;
    std::result::Result::Ok(ImageMagickCommand {
        args,
        expected_output_path: output_path.to_path_buf(),
    })
}

/// Adds position arguments for modern ImageMagick syntax
fn add_modern_position_args(args: &mut std::vec::Vec<std::string::String>, position: &crate::schemas::watermark_schemas::WatermarkPosition) {
    use crate::schemas::watermark_schemas::{WatermarkPosition, CornerPosition, EdgePosition};
    
    match position {
        WatermarkPosition::Corner(corner) => {
            let gravity = match corner {
                CornerPosition::TopLeft => "northwest",
                CornerPosition::TopRight => "northeast",
                CornerPosition::BottomLeft => "southwest",
                CornerPosition::BottomRight => "southeast",
            };
            args.push(std::string::String::from("-gravity"));
            args.push(std::string::String::from(gravity));
            args.push(std::string::String::from("-geometry"));
            args.push(std::string::String::from("+10+10")); // 10px offset from corner
        }
        WatermarkPosition::Edge(edge) => {
            let gravity = match edge {
                EdgePosition::Top => "north",
                EdgePosition::Bottom => "south",
                EdgePosition::Left => "west",
                EdgePosition::Right => "east",
            };
            args.push(std::string::String::from("-gravity"));
            args.push(std::string::String::from(gravity));
            args.push(std::string::String::from("-geometry"));
            args.push(std::string::String::from("+0+10")); // 10px offset from edge
        }
        WatermarkPosition::Center => {
            args.push(std::string::String::from("-gravity"));
            args.push(std::string::String::from("center"));
        }
        WatermarkPosition::Custom { x_percent, y_percent } => {
            // Custom positioning using geometry
            let geometry = std::format!("+{}%+{}%", x_percent, y_percent);
            args.push(std::string::String::from("-geometry"));
            args.push(geometry);
        }
    }
}


/// Executes ImageMagick command with timeout handling
async fn execute_imagemagick_command_with_timeout(
    command: crate::services::watermarking::imagemagick_commands::ImageMagickCommand,
) -> std::result::Result<(), WatermarkError> {
    let timeout_duration = std::time::Duration::from_secs(PROCESSING_TIMEOUT_SECONDS);
    
    log::info!("Starting ImageMagick command with {} second timeout", PROCESSING_TIMEOUT_SECONDS);
    let command_future = execute_imagemagick_command(command);
    
    match tokio::time::timeout(timeout_duration, command_future).await {
        std::result::Result::Ok(result) => {
            match &result {
                std::result::Result::Ok(_) => log::info!("ImageMagick command completed successfully"),
                std::result::Result::Err(e) => log::error!("ImageMagick command failed: {}", e),
            }
            result
        },
        std::result::Result::Err(_) => {
            log::error!("ImageMagick command timed out after {} seconds", PROCESSING_TIMEOUT_SECONDS);
            std::result::Result::Err(WatermarkError::ProcessingTimeout)
        },
    }
}

/// Executes ImageMagick command and handles errors
async fn execute_imagemagick_command(
    command: crate::services::watermarking::imagemagick_commands::ImageMagickCommand,
) -> std::result::Result<(), WatermarkError> {
    log::info!("Executing ImageMagick composite command: composite {}", command.args.join(" "));
    let mut cmd = tokio::process::Command::new("composite");
    cmd.args(&command.args);
    
    // Preserve color profiles and quality
    cmd.env("MAGICK_PRESERVE_COLORMAP", "true");
    
    let output = cmd.output().await?;
    
    if !output.status.success() {
        let stderr = std::string::String::from_utf8_lossy(&output.stderr);
        let stdout = std::string::String::from_utf8_lossy(&output.stdout);
        log::error!("ImageMagick command failed with status: {}", output.status);
        log::error!("Command was: magick {}", command.args.join(" "));
        log::error!("STDOUT: {}", stdout);
        log::error!("STDERR: {}", stderr);
        return std::result::Result::Err(WatermarkError::ImageProcessing(
            std::format!("ImageMagick composite command failed")
        ));
    }
    
    std::result::Result::Ok(())
}

/// Generates filename for watermarked asset
fn generate_watermarked_filename(original_name: &str) -> std::string::String {
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    
    // Always use PNG for watermarked images to avoid compression artifacts that darken the image
    if let std::option::Option::Some(extension_pos) = original_name.rfind('.') {
        let name_part = &original_name[..extension_pos];
        std::format!("{}_watermarked_{}.png", name_part, timestamp)
    } else {
        std::format!("{}_watermarked_{}.png", original_name, timestamp)
    }
}

/// Gets content type from filename extension
fn get_content_type_from_filename(filename: &str) -> std::string::String {
    if filename.ends_with(".png") {
        std::string::String::from("image/png")
    } else if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
        std::string::String::from("image/jpeg")
    } else if filename.ends_with(".webp") {
        std::string::String::from("image/webp")
    } else if filename.ends_with(".svg") {
        std::string::String::from("image/svg+xml")
    } else {
        std::string::String::from("image/png") // Use PNG as default to avoid compression artifacts
    }
}

/// Creates a new asset record for the watermarked image
async fn create_watermarked_asset(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    name: &str,
    gcs_object_name: &str,
    url: &str,
    source_asset: &crate::db::assets::Asset,
) -> std::result::Result<crate::db::assets::Asset, WatermarkError> {
    let row = sqlx::query!(
        r#"
        INSERT INTO assets (user_id, name, type, gcs_object_name, url, collection_id, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, user_id, name, type, gcs_object_name, url, created_at, updated_at, collection_id, metadata
        "#,
        user_id,
        name,
        source_asset.r#type,
        gcs_object_name,
        url,
        source_asset.collection_id,
        source_asset.metadata
    )
    .fetch_one(pool)
    .await?;
    
    let watermarked_asset = crate::db::assets::Asset {
        id: row.id,
        user_id: row.user_id,
        name: row.name,
        r#type: row.r#type,
        gcs_object_name: row.gcs_object_name,
        url: row.url,
        created_at: std::option::Option::Some(row.created_at),
        updated_at: std::option::Option::Some(row.updated_at),
        collection_id: row.collection_id,
        metadata: row.metadata,
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

    #[test]
    fn test_generate_watermarked_filename() {
        let original = "logo.png";
        let result = generate_watermarked_filename(original);
        assert!(result.starts_with("logo_watermarked_"));
        assert!(result.ends_with(".png"));
    }

    #[test]
    fn test_generate_watermarked_filename_no_extension() {
        let original = "image_file";
        let result = generate_watermarked_filename(original);
        assert!(result.starts_with("image_file_watermarked_"));
    }

    #[test]
    fn test_get_content_type_from_filename() {
        assert_eq!(get_content_type_from_filename("test.png"), "image/png");
        assert_eq!(get_content_type_from_filename("test.jpg"), "image/jpeg");
        assert_eq!(get_content_type_from_filename("test.jpeg"), "image/jpeg");
        assert_eq!(get_content_type_from_filename("test.webp"), "image/webp");
        assert_eq!(get_content_type_from_filename("test.svg"), "image/svg+xml");
        assert_eq!(get_content_type_from_filename("test.unknown"), "image/png");
    }

    #[test]
    fn test_validate_image_assets() {
        let image_asset = crate::db::assets::Asset {
            id: uuid::Uuid::new_v4(),
            user_id: uuid::Uuid::new_v4(),
            name: std::string::String::from("test.png"),
            r#type: std::string::String::from("image/png"),
            gcs_object_name: std::string::String::from("test.png"),
            url: std::string::String::from("https://example.com/test.png"),
            created_at: std::option::Option::Some(chrono::Utc::now()),
            updated_at: std::option::Option::Some(chrono::Utc::now()),
            collection_id: std::option::Option::None,
            metadata: std::option::Option::None,
        };

        let video_asset = crate::db::assets::Asset {
            id: uuid::Uuid::new_v4(),
            user_id: uuid::Uuid::new_v4(),
            name: std::string::String::from("test.mp4"),
            r#type: std::string::String::from("video/mp4"),
            gcs_object_name: std::string::String::from("test.mp4"),
            url: std::string::String::from("https://example.com/test.mp4"),
            created_at: std::option::Option::Some(chrono::Utc::now()),
            updated_at: std::option::Option::Some(chrono::Utc::now()),
            collection_id: std::option::Option::None,
            metadata: std::option::Option::None,
        };

        // Valid case: both assets are images
        assert!(validate_image_assets(&image_asset, &image_asset).is_ok());

        // Invalid case: source is not an image
        assert!(validate_image_assets(&video_asset, &image_asset).is_err());

        // Invalid case: logo is not an image
        assert!(validate_image_assets(&image_asset, &video_asset).is_err());
    }

    #[tokio::test]
    async fn test_file_size_validation() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_file_size.txt");
        
        // Create a small file (should pass)
        tokio::fs::write(&test_file, b"small content").await.unwrap();
        assert!(validate_file_size(&test_file).await.is_ok());
        
        // Clean up
        let _ = tokio::fs::remove_file(&test_file).await;
    }

    #[test]
    fn test_resource_limits_constants() {
        // Verify our constants are reasonable
        assert_eq!(MAX_FILE_SIZE_BYTES, 2 * 1024 * 1024 * 1024); // 2GB
        assert_eq!(PROCESSING_TIMEOUT_SECONDS, 5 * 60); // 5 minutes
    }
}
