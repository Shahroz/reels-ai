use serde::Serialize;
use serde_json;
use serde_yaml;
use thiserror::Error;
use tracing::instrument;

/// Specifies the desired serialization format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationFormat {
    Json,
    Yaml,
}

/// Represents errors that can occur during serialization.
#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("YAML serialization error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

/// Serializes the given data into the specified format (JSON or YAML).
///
/// This function can be used to serialize 'few shots' data structures or any other
/// data that implements `serde::Serialize`.
///
/// # Arguments
///
/// * `data` - The data to serialize. Must implement `serde::Serialize`.
/// * `format` - The desired output format (`SerializationFormat::Json` or `SerializationFormat::Yaml`).
///
/// # Returns
///
/// A `Result` containing the serialized string or a `SerializationError`.
#[instrument(skip(data))]
pub fn serialize_data<T: Serialize>(data: &T, format: SerializationFormat) -> Result<String, SerializationError> {
    match format {
        SerializationFormat::Json => {
            // Using pretty print for JSON for better readability
            serde_json::to_string_pretty(data).map_err(SerializationError::Json)
        }
        SerializationFormat::Yaml => {
            serde_yaml::to_string(data).map_err(SerializationError::Yaml)
        }
    }
}

// Potential future enhancement: Add specific functions tailored to
// FewShotsOutput or FewShotsOutputs if needed, which might call serialize_data internally.
