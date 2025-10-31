//! Defines the request payload for editing a creative.
//!
//! This struct is used to pass instructions for modifying an existing creative's
//! HTML content or other creative attributes. It contains a single field, `instruction`,
//! which specifies the desired changes in a textual format. The request is
//! typically deserialized from JSON and handled by an endpoint like `edit_creative`.

// No `use` statements as per rust_guidelines.md.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct EditCreativeRequest {
    #[schema(example = "Make the header blue")]
    pub instruction: std::string::String,
    #[schema(example = "<html><body><h1>My New Content</h1></body></html>", value_type=Option<String>)]
    pub html_content: Option<std::string::String>,
}