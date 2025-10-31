//! Handles a raw TOML string response from an LLM, attempting to parse, validate, and deserialize it.
//!
//! This function takes a raw string, which is expected to be TOML content,
//! along with necessary context like the prompt, timestamp, BPE tokenizer,
//! input token count, and a compiled JSON schema for validation.
//! It uses `hacky_toml_loads` for parsing, converts the TOML to JSON,
//! validates against the schema using `validate_value_unified`, and then
//! attempts to deserialize the validated JSON into the target type `T`.
//! Logs the outcome of these operations. This function is an internal helper.

//! Revision History
//! - 2025-05-16T14:57:33Z @AI: Extracted from crates/llm/src/llm_typed_unified/mod.rs and adapted to guidelines.
//! - 2025-05-16T15:08:36Z @AI: Standardized to use detailed LlmTypedLog and new signature.

// Fully qualified paths as per rust_guidelines, except for items typically in the prelude.
// `LlmTypedLog` is assumed to be accessible via `crate::llm::llm_typed_unified::llm_typed_log::LlmTypedLog`.

#[allow(clippy::too_many_arguments)] // Existing function signature has many arguments.
pub(super) fn handle_raw_toml_response<T>(
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
        // 1. Attempt to parse TOML
        let parsed_toml_value =
            crate::hacky_toml_loads::hacky_toml_loads(raw_response_content).ok_or_else(|| {
                anyhow::anyhow!(
                    "Failed to parse TOML from raw response using hacky_toml_loads. Raw content snippet (first 100 chars): '{}'",
                    raw_response_content.chars().take(100).collect::<String>()
                )
           })?;

       // 2. Convert TOML Value to JSON Value
        let json_value_from_toml = crate::llm_typed_unified::toml_value_to_json_value::toml_value_to_json_value(
           &parsed_toml_value, // Clone for potential error reporting if conversion fails
        )
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to convert TOML value to JSON value: {}. Original TOML value: {:?}",
                e,
                parsed_toml_value // Log the problematic TOML value
            )
        })?;
        
       final_response_payload_for_log = json_value_from_toml.clone(); // Update log payload

       // 3. Validate the JSON Value against the provided schema
        let validated_json_value = crate::llm_typed_unified::validate_value_unified::validate_value_unified(
            compiled_schema,
            &json_value_from_toml, // Pass reference to the JSON value
        )
            .map_err(|e| {
                anyhow::anyhow!(
                    "Schema validation failed for JSON derived from TOML: {}. JSON value: {}",
                    e,
                    json_value_from_toml // Log the JSON value that failed validation
                )
            })?;
        
        final_response_payload_for_log = validated_json_value.clone(); // Update log payload

        // 4. Deserialize the validated JSON Value into the target type T
        serde_json::from_value::<T>(validated_json_value.clone()).map_err(|e| {
            anyhow::anyhow!(
                "Failed to deserialize validated JSON value (from TOML) to target type: {}. Validated JSON value: {}",
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
        crate::llm_typed_unified::write_log::write_log(&log_entry, log_dir, &log_file_name)
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
