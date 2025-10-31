//! Defines the structure for inline data, used in multimodal prompts for the Gemini API.
//!
//! This struct is part of a `Part` when sending data like images directly in the request.
//! It specifies the MIME type and the base64-encoded data.
//! Adheres to one-item-per-file and uses fully qualified paths.

/// Represents inline data for a multimodal request part.
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq)]
pub struct InlineData {
    /// The MIME type of the data (e.g., "image/jpeg", "image/png").
    #[serde(rename = "mimeType")]
    pub mime_type: std::string::String,
    /// The base64-encoded data.
    pub data: std::string::String,
}

#[cfg(test)]
mod tests {
    //! Tests for the InlineData struct.

    #[test]
    fn test_inline_data_creation() {
        //! Verifies that an InlineData struct can be created.
        let inline_data = super::InlineData {
            mime_type: std::string::String::from("image/png"),
            data: std::string::String::from("base64encodedstring"),
        };
        std::assert_eq!(inline_data.mime_type, "image/png");
        std::assert_eq!(inline_data.data, "base64encodedstring");
    }
}