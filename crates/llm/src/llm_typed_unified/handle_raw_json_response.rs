//! Handles the raw JSON response from an LLM call, including parsing, validation, and deserialization.
//!
//! This function attempts to parse a raw string response as JSON, validates it against a
//! provided JSON schema, and then deserializes it into the target type `T`.
//! It uses a detailed logging mechanism, similar to `handle_raw_yaml_response`.
//!
//! Revision History
//! - 2025-05-16T15:08:36Z @AI: Standardized to use detailed LlmTypedLog and new signature.

// Note: Fully qualified paths are used as per guidelines.
// `String`, `Option`, `usize` are used directly as they are typically in prelude or unambiguous.
// `serde::Serialize` is used directly instead of an alias.

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_raw_json_response<T>(
    raw_response_content: &str,
    compiled_schema: &jsonschema::JSONSchema,
    // Logging related parameters
    request_id: std::option::Option<std::string::String>,
    current_timestamp_str: std::string::String, // Pre-formatted timestamp string
    model_name: std::string::String,
    prompt_tokens: std::option::Option<u32>,
    llm_reported_completion_tokens: std::option::Option<u32>, // Calculated from raw response
    llm_reported_total_tokens: std::option::Option<u32>,    // Sum of prompt and completion
    request_payload_for_log: &serde_json::Value, // Reference, will be cloned for log
    log_dir: &std::path::Path,
    log_file_name_prefix: &str,
    processing_start_time: std::time::Instant,
) -> anyhow::Result<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned + schemars::JsonSchema + crate::few_shots_traits::FewShotsOutput<T>,
{
    let mut final_response_payload_for_log =
        serde_json::Value::String(raw_response_content.to_string());
    let mut error_for_log: std::option::Option<std::string::String> = None;

    let outcome: anyhow::Result<T> = (|| {
        // 1. Attempt to parse JSON (potentially with workarounds for common LLM quirks)
        let parsed_json_value =
            crate::hacky_json_loads::hacky_json_loads(raw_response_content).ok_or_else(|| {
                anyhow::anyhow!("Failed to parse JSON from raw response using hacky_json_loads. Raw content snippet (first 100 chars): '{}'", raw_response_content.chars().take(100).collect::<String>())
            })?;
        
        final_response_payload_for_log = parsed_json_value.clone(); // Update log payload

        // 2. Validate the JSON Value against the provided schema
        let validated_json_value =
            super::validate_value_unified::validate_value_unified(
                compiled_schema,
                &parsed_json_value, // Pass reference to the JSON value
            )
            .map_err(|e| {
                anyhow::anyhow!(
                    "Schema validation failed for JSON: {}. JSON value: {}",
                    e,
                    parsed_json_value // Log the JSON value that failed validation
                )
            })?;
        
        // Update log payload with the (potentially modified by validation) validated value
        final_response_payload_for_log = validated_json_value.clone();

        // 3. Deserialize the validated JSON Value into the target type T
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
    let log_entry = super::llm_typed_log::LlmTypedLog {
        request_id: request_id.clone(),
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

    let sanitized_timestamp = log_entry.timestamp.replace([':', '.'], "-").replace('+', "ZPLUS");
    let sanitized_request_id = request_id.as_deref().unwrap_or("unknown").replace(['/', '\\', ':', '*','?', '"', '<', '>', '|'], "_");
    
    let log_file_name = std::format!(
        "{}_{}_{}.yaml",
        log_file_name_prefix,
        sanitized_timestamp,
        sanitized_request_id
    );

    if let Err(log_write_err) =
        super::write_log::write_log(&log_entry, log_dir, &log_file_name)
    {
        // Using println! for simplicity.
        println!(
            "Warning: Failed to write LLM interaction log to file '{}': {}",
            log_dir.join(&log_file_name).display(),
            log_write_err
        );
    }

    outcome
}
