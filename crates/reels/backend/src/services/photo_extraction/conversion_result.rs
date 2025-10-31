//! Represents the result of RAW image format conversion.
//!
//! This struct contains all the metadata about a successfully converted RAW image,
//! including the new file location, content type, and format information.
//! Used by both HEIC and DNG conversion functions to return conversion details.

/// Result of RAW image format conversion
#[derive(Debug, Clone)]
pub struct ConversionResult {
    /// The new image object name in GCS
    pub new_object_name: std::string::String,
    /// The public URL of the converted image file
    pub new_public_url: std::string::String,
    /// The content type (e.g. "image/webp", "image/png")
    pub new_content_type: std::string::String,
    /// The file extension (e.g. "webp", "png")  
    pub new_extension: std::string::String,
    /// The output format used for conversion
    pub output_format: crate::services::photo_extraction::output_format::OutputFormat,
}

impl ConversionResult {
    /// Creates a new ConversionResult with the provided details
    pub fn new(
        new_object_name: std::string::String,
        new_public_url: std::string::String,
        new_content_type: std::string::String,
        new_extension: std::string::String,
        output_format: crate::services::photo_extraction::output_format::OutputFormat,
    ) -> Self {
        Self {
            new_object_name,
            new_public_url,
            new_content_type,
            new_extension,
            output_format,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_conversion_result_creation() {
        let result = super::ConversionResult::new(
            std::string::String::from("user123/image.webp"),
            std::string::String::from("https://storage.googleapis.com/bucket/user123/image.webp"),
            std::string::String::from("image/webp"),
            std::string::String::from("webp"),
            crate::services::photo_extraction::output_format::OutputFormat::WebP,
        );

        assert_eq!(result.new_extension, "webp");
        assert_eq!(result.new_content_type, "image/webp");
        assert!(result.new_object_name.ends_with(".webp"));
        assert_eq!(result.output_format, crate::services::photo_extraction::output_format::OutputFormat::WebP);
    }
} 