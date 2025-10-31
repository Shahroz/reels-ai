//! Represents a field that can be either a single string or an array of strings.
//!
//! This enum is used for parameters like 'stop' in OpenAI requests,
//! which can accept either a single stop sequence or multiple.
//! Facilitates correct serialization based on the provided value.
//! Helps handle flexible API parameters.

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)] // Important for correct serialization/deserialization
pub enum StringOrArray {
    String(String),
    Array(Vec<String>),
}
