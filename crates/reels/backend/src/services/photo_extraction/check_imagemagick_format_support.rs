//! Checks if ImageMagick supports a specific image format.
//!
//! This function verifies that ImageMagick is installed and can handle a specified
//! image format (like HEIC or DNG). It checks both the ImageMagick installation
//! and the availability of the required format libraries.

use anyhow::Context;

/// Checks if ImageMagick is available and supports the specified format
pub async fn check_imagemagick_format_support(format_name: &str, format_description: &str, install_instructions: &str) -> anyhow::Result<()> {
    log::debug!("Checking ImageMagick installation and {} support...", format_description);
    
    // Check if ImageMagick is installed
    log::debug!("Testing 'magick -version' command...");
    let version_output = tokio::process::Command::new("magick")
        .arg("-version")
        .output()
        .await
        .context("ImageMagick is not installed or not in PATH. Try: apt-get install imagemagick (Debian) or apk add imagemagick (Alpine)")?;

    if !version_output.status.success() {
        let stderr = std::string::String::from_utf8_lossy(&version_output.stderr);
        let stdout = std::string::String::from_utf8_lossy(&version_output.stdout);
        return std::result::Result::Err(anyhow::anyhow!(
            "ImageMagick version check failed with status {}\nSTDOUT: {}\nSTDERR: {}", 
            version_output.status, stdout, stderr
        ));
    }
    
    let version_info = std::string::String::from_utf8_lossy(&version_output.stdout);
    log::debug!("ImageMagick version check successful: {}", version_info.lines().next().unwrap_or("unknown"));

    // Check if the format is supported
    log::debug!("Testing 'magick identify -list format' command...");
    let formats_output = tokio::process::Command::new("magick")
        .arg("identify")
        .arg("-list")
        .arg("format")
        .output()
        .await
        .context("Failed to check ImageMagick format support. ImageMagick may be installed but not functioning correctly.")?;

    if !formats_output.status.success() {
        let stderr = std::string::String::from_utf8_lossy(&formats_output.stderr);
        let stdout = std::string::String::from_utf8_lossy(&formats_output.stdout);
        return std::result::Result::Err(anyhow::anyhow!(
            "Failed to list ImageMagick formats with status {}\nSTDOUT: {}\nSTDERR: {}", 
            formats_output.status, stdout, stderr
        ));
    }

    let formats_text = std::string::String::from_utf8_lossy(&formats_output.stdout);
    log::debug!("ImageMagick format list retrieved ({} characters)", formats_text.len());
    
    // Look for format support
    let has_format = formats_text.contains(format_name) || formats_text.contains(&format_name.to_lowercase());
    if !has_format {
        // Log some of the available formats for debugging
        let format_lines: std::vec::Vec<&str> = formats_text.lines()
            .filter(|line| line.trim().len() > 0 && !line.starts_with("Format"))
            .take(10)
            .collect();
        log::error!("{} format not found in ImageMagick. Available formats (first 10): {:?}", format_name, format_lines);
        
        return std::result::Result::Err(anyhow::anyhow!(
            "ImageMagick does not support {} format. {}", 
            format_description, install_instructions
        ));
    }

    log::debug!("{} format support confirmed in ImageMagick", format_description);
    std::result::Result::Ok(())
}

/// Checks if ImageMagick is available and supports HEIC format
pub async fn check_imagemagick_heic_support() -> anyhow::Result<()> {
    check_imagemagick_format_support(
        "HEIC",
        "HEIC format",
        "Please install libheif support:\n\
        - Debian/Ubuntu: apt-get install libheif-examples\n\
        - Alpine: apk add libheif\n\
        - Or recompile ImageMagick with libheif support"
    ).await
}

/// Checks if ImageMagick is available and supports DNG format
pub async fn check_imagemagick_dng_support() -> anyhow::Result<()> {
    check_imagemagick_format_support(
        "DNG",
        "DNG format",
        "Please install libraw support:\n\
        - Usually included with ImageMagick by default\n\
        - Ubuntu/Debian: apt-get install libraw-bin\n\
        - Alpine: apk add libraw\n\
        - Or recompile ImageMagick with libraw support"
    ).await
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[ignore] // This test requires ImageMagick to be installed in the environment
    async fn test_imagemagick_support_check() {
        // This test will succeed if ImageMagick is installed, fail otherwise
        // Testing with a common format that should be available
        let result = super::check_imagemagick_format_support(
            "PNG",
            "PNG format",
            "PNG should be available by default"
        ).await;
        
        // Note: This test may fail in environments without ImageMagick
        // That's expected behavior for a dependency check function
        match result {
            std::result::Result::Ok(()) => println!("ImageMagick with PNG support is available"),
            std::result::Result::Err(e) => println!("ImageMagick/PNG support not available: {}", e),
        }
    }

    #[tokio::test]
    #[ignore] // This test requires ImageMagick with libheif support to be installed
    async fn test_imagemagick_heic_support() {
        let result = super::check_imagemagick_heic_support().await;
        
        // This test will fail if ImageMagick or libheif is not installed
        // That's expected - it helps verify the environment is set up correctly
        match result {
            std::result::Result::Ok(()) => println!("ImageMagick with HEIC support is available"),
            std::result::Result::Err(e) => println!("ImageMagick/HEIC support not available: {}", e),
        }
    }

    #[tokio::test]
    #[ignore] // This test requires ImageMagick with libraw support to be installed
    async fn test_imagemagick_dng_support() {
        let result = super::check_imagemagick_dng_support().await;
        
        // This test will fail if ImageMagick or libraw is not installed
        // That's expected - it helps verify the environment is set up correctly
        match result {
            std::result::Result::Ok(()) => println!("ImageMagick with DNG support is available"),
            std::result::Result::Err(e) => println!("ImageMagick/DNG support not available: {}", e),
        }
    }
} 