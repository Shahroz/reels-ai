//! Defines output formats for RAW image conversion.
//!
//! This enum represents the supported output formats when converting RAW images
//! like HEIC and DNG to web-compatible formats. WebP is the default format for
//! optimal compression, while PNG provides universal compatibility.

/// Supported output formats for RAW image conversion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// PNG format - larger file size, universal compatibility
    Png,
    /// WebP format - smaller file size, modern browser support (default)
    WebP,
}

impl OutputFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Png => "png",
            OutputFormat::WebP => "webp",
        }
    }

    /// Get the MIME type for this format
    pub fn content_type(&self) -> &'static str {
        match self {
            OutputFormat::Png => "image/png",
            OutputFormat::WebP => "image/webp",
        }
    }

    /// Get the ImageMagick format specifier
    pub fn imagemagick_format(&self) -> &'static str {
        match self {
            OutputFormat::Png => "png",
            OutputFormat::WebP => "webp",
        }
    }
}

impl std::default::Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::WebP
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_png_properties() {
        let format = super::OutputFormat::Png;
        assert_eq!(format.extension(), "png");
        assert_eq!(format.content_type(), "image/png");
        assert_eq!(format.imagemagick_format(), "png");
    }

    #[test]
    fn test_webp_properties() {
        let format = super::OutputFormat::WebP;
        assert_eq!(format.extension(), "webp");
        assert_eq!(format.content_type(), "image/webp");
        assert_eq!(format.imagemagick_format(), "webp");
    }

    #[test]
    fn test_default_format() {
        let format = super::OutputFormat::default();
        assert_eq!(format, super::OutputFormat::WebP);
    }
} 