//! Validates a serde_json::Value against a compiled jsonschema::JSONSchema.
//!
//! This function is an internal helper responsible for ensuring that a given JSON value,
//! typically received from an LLM, adheres to a predefined schema structure.
//! It offers a specific fallback for arrays: if the entire array fails validation
//! but its initial element is valid, that element is accepted. This handles cases
//! where an LLM might wrap a valid single object in an unnecessary list.
//!
//! Revision History
//! - 2025-05-16T14:52:28Z @AI: Extracted from crates/llm/src/llm_typed_unified/mod.rs and adapted to guidelines.

// This function is not public as it's an internal helper.
// It adheres to the "one item per file" and "fully qualified paths" guidelines.
pub fn validate_value_unified(
    compiled_schema: &jsonschema::JSONSchema,
    response_value: &serde_json::Value,
) -> anyhow::Result<serde_json::Value> {
    match compiled_schema.validate(response_value) {
        std::result::Result::Ok(_) => std::result::Result::Ok(response_value.clone()),
        std::result::Result::Err(validation_errors) => {
            if response_value.is_array() {
                if let Some(first_element) = response_value.as_array().and_then(|arr| arr.first()) {
                    if compiled_schema.validate(first_element).is_ok() {
                        return std::result::Result::Ok(first_element.clone());
                    }
                }
            }
            // Vec and String are typically in the prelude, so direct use is permitted by guidelines.
            let error_messages: Vec<String> = validation_errors
                .map(|e| std::format!("{}", e)) // std::format! for FQP
                .collect();
            std::result::Result::Err(anyhow::anyhow!(
                "Response JSON does not conform to the provided schema. Errors: [{}]. Value: {}",
                error_messages.join(", "),
                response_value
            ))
        }
    }
}
