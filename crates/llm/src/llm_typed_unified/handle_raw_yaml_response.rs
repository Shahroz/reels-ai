//! Defines the `handle_raw_yaml_response` function for processing LLM YAML outputs.
//!
//! This internal helper function orchestrates parsing of potentially malformed YAML,
//! conversion to JSON, schema validation, and deserialization into a target type.
//! It encapsulates error handling and provides detailed logging of the interaction.
//! Designed for use within the `llm_typed_unified` module to standardize response processing.
//!
//! Revision History
//! - 2025-05-16T14:55:45Z @AI: Initial creation of `handle_raw_yaml_response` function and file.

// Note: This function is not public, as it's an internal helper for the llm_typed_unified module.
// Adherence to "one item per file" and "fully qualified paths" from rust_guidelines.
#[allow(clippy::too_many_arguments)] // Common for comprehensive helper functions
pub(super) fn handle_raw_yaml_response<T: serde::de::DeserializeOwned>(
    raw_response_content: &str,
    compiled_schema: &jsonschema::JSONSchema,
    // Logging related parameters
    request_id: std::option::Option<std::string::String>,
    current_timestamp_str: std::string::String, // Pre-formatted timestamp string
    model_name: std::string::String,
    prompt_tokens: std::option::Option<u32>,
    llm_reported_completion_tokens: std::option::Option<u32>,
    llm_reported_total_tokens: std::option::Option<u32>,
    request_payload_for_log: &serde_json::Value, // Reference, will be cloned for log
    log_dir: &std::path::Path,
    log_file_name_prefix: &str,
    processing_start_time: std::time::Instant,
) -> anyhow::Result<T> {
    let mut final_response_payload_for_log =
        serde_json::Value::String(raw_response_content.to_string());
    let mut error_for_log: std::option::Option<std::string::String> = None;

    let outcome: anyhow::Result<T> = (|| {
        // 1. Attempt to parse YAML (potentially with workarounds for common LLM quirks)
        let yaml_value = crate::hacky_yaml_loads::hacky_yaml_loads(raw_response_content)
            .ok_or_else(|| {
                anyhow::anyhow!(
                   "Failed to parse YAML from raw response using hacky_yaml_loads. Raw content snippet (first 100 chars): '{}'",
                    raw_response_content.chars().take(100).collect::<String>()
                )
            })?;

        // 2. Convert YAML Value to JSON Value
        let json_value_from_yaml =
            crate::llm_typed_unified::yaml_value_to_json_value::yaml_value_to_json_value(
                yaml_value.clone(), // Clone for potential error reporting if conversion fails
            )
                .map_err(|e| {
                    anyhow::anyhow!(
                    "Failed to convert YAML value to JSON value: {}. Original YAML value: {:?}",
                    e,
                    yaml_value // Log the problematic YAML value
                )
                })?;

        final_response_payload_for_log = json_value_from_yaml.clone(); // Update log payload

        // 3. Validate the JSON Value against the provided schema
        let validated_json_value = crate::llm_typed_unified::validate_value_unified::validate_value_unified(
            compiled_schema,
            &json_value_from_yaml, // Pass reference to the JSON value
        )
            .map_err(|e| {
                anyhow::anyhow!(
                "Schema validation failed for JSON derived from YAML: {}. JSON value: {}",
                e,
                json_value_from_yaml // Log the JSON value that failed validation
            )
            })?;

        // Update log payload with the (potentially modified by validation) validated value
        final_response_payload_for_log = validated_json_value.clone();

        // 4. Deserialize the validated JSON Value into the target type T
        serde_json::from_value::<T>(validated_json_value.clone()).map_err(|e| {
            anyhow::anyhow!(
                "Failed to deserialize validated JSON value to target type: {}. Validated JSON value: {}",
                e,
                validated_json_value // Log the JSON value that failed deserialization
            )
        })
    })(); // End of IIFE-like closure for error handling with `?`

    if let Err(e) = &outcome {
        error_for_log = Some(std::format!("{}", e));
    }

    // Calculate processing duration
    let duration_ms = std::option::Option::Some(processing_start_time.elapsed().as_millis() as u64);

    // Prepare the log entry
    let log_entry = crate::llm_typed_unified::llm_typed_log::LlmTypedLog {
        request_id: request_id.clone(), // Clone Option<String>
        timestamp: current_timestamp_str,
        model_name,
        prompt_tokens,
        completion_tokens: llm_reported_completion_tokens,
        total_tokens: llm_reported_total_tokens,
        request_payload: request_payload_for_log.clone(),
        response_payload: final_response_payload_for_log,
        error_message: error_for_log,
        duration_ms,
    };

    // Construct a unique log file name
    // Sanitize timestamp and request_id for filename characters
    let sanitized_timestamp = log_entry.timestamp.replace([':', '.'], "-").replace('+', "ZPLUS");
    let sanitized_request_id = request_id.as_deref().unwrap_or("unknown").replace(['/', '\\', ':', '*','?', '"', '<', '>', '|'], "_");

    let log_file_name = std::format!(
        "{}_{}_{}.yaml",
        log_file_name_prefix,
        sanitized_timestamp,
        sanitized_request_id
    );

    // Write the log entry to a file
    // Errors from write_log are converted to String and ignored here, as the primary function outcome is already determined.
    // In a real scenario, logging failures might be handled more robustly (e.g.,eprintln).
    if let Err(log_write_err) =
        crate::llm_typed_unified::write_log::write_log(&log_entry, log_dir, &log_file_name)
    {
        // Using println! for simplicity, as per guidelines (available without FQN)
        // A more sophisticated logging mechanism for meta-errors might be used in a larger app.
        println!(
            "Warning: Failed to write LLM interaction log to file '{}': {}",
            log_dir.join(&log_file_name).display(),
            log_write_err
        );
    }

    outcome // Return the result of parsing/validation/deserialization
}