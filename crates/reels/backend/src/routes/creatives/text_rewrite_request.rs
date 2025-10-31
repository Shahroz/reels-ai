//! Defines the request payload for the text rewrite endpoint.
//!
//! This struct is used to pass the original text and an instruction for how
//! to modify it. The request is typically deserialized from JSON.

// Adhering to rust_guidelines.md: No `use` statements. Fully qualified paths will be used if needed.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct TextRewriteRequest {
    #[schema(example = "<html><body><p>This is some old text.</p></body></html>", value_type=String)]
    pub text: std::string::String,
    #[schema(example = "Make the text more concise and friendly.", value_type=String)]
    pub instruction: std::string::String,
}