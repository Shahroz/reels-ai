//! Defines the expected format type of the OpenAI response.
//!
//! This enum specifies whether the model's response should be plain text
//! or formatted as a JSON object. Used within ResponseFormat.
//! Maps to OpenAI's `response_format` options.
//! Serialization uses lowercase names.

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")] // Assuming rename_all applies, adjust if not
pub enum ResponseType {
    Text,
    #[serde(rename = "json_object")] // Explicit rename often needed for nested types in OpenAI
    JSON, // Keep original name JSON, map via rename
}
