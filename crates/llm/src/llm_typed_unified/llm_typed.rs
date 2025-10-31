//! Provides the primary `llm_typed` function for making structured calls to LLMs.
//!
//! This function orchestrates interactions with various LLM vendors, handling
//! schema generation, prompt construction, retries, and response parsing for
//! different output formats (JSON, YAML, TOML). It aims to provide a unified
//! interface for obtaining typed data from LLMs, with detailed logging.
//!
//! Revision History
//! - 2025-05-16T15:08:36Z @AI: Extracted from mod.rs and adapted to use detailed logging.

// Adhering to rust_guidelines: FQPs for non-prelude items.
// `use` statements for external crates are generally acceptable if followed by FQP usage internally.

// External Crate Imports (used with FQPs or as per common practice)
use anyhow::Context;
// use jsonschema::JSONSchema;
// use schemars::JsonSchema;
// use serde::{de::DeserializeOwned, Serialize as SerdeSerializeTrait};
// use serde_json;
// use tiktoken_rs; // For CoreBPE
// use tokio; // For time::sleep, time::Duration

// Current Crate Imports (using super:: or crate::)
// use crate::few_shots_traits::FewShotsOutput;
// use crate::vendors::claude::{ClaudeContentBlock, ClaudeMessage, ClaudeMessageRequest};
// use crate::vendors::gemini::completion::generate_gemini_response;
// use crate::vendors::openai::call_gpt_with_body::call_gpt_with_body;
// use crate::vendors::openai::message::Message as OpenAIMessage;
// use crate::vendors::openai::openai_chat_completion_request::OpenAIChatCompletionRequest;
// use crate::vendors::openai::role::Role as OpenAIRole;
// use crate::vendors::replicate::call_replicate_api;

use crate::vendors::openai::response_type::ResponseType;

/// Unified typed LLM call that attempts requests across multiple vendor models with retries.
///
/// This function manages the entire lifecycle of a typed LLM request:
/// 1. Generates a JSON schema for the target type `T`.
/// 2. Constructs a detailed prompt including the schema, few-shot examples, and the user's query.
/// 3. Iterates through a list of `VendorModel`s, making API calls with retries.
/// 4. For each response, it attempts to parse (JSON, YAML, TOML), validate against the schema,
///    and deserialize into type `T`.
/// 5. Logs interactions using the detailed `LlmTypedLog` structure.
///
/// # Arguments
/// * `prompt`: The user's core query or instruction for the LLM.
/// * `models`: A `Vec` of `VendorModel` enums specifying which LLMs to try in order.
/// * `retries`: The number of retries per model before giving up on that model.
/// * `format`: An `Option<OutputFormat>` specifying the desired output format from the LLM.
///             Defaults to `OutputFormat::Json` if `None`.
/// * `debug_mode`: If `true`, prints the full prompt and raw responses to stdout.
///
/// # Returns
/// * `anyhow::Result<T>`: On success, the deserialized data of type `T`. On failure
///   (e.g., all models/retries exhausted, parsing/validation errors), an `anyhow::Error`.
#[allow(clippy::too_many_lines)] // This function is complex and orchestrates many steps.
#[allow(clippy::too_many_arguments)] // Reflects the number of parameters needed for flexible LLM calls.
pub async fn llm_typed<T>(
    prompt: std::string::String,
    models: std::vec::Vec<super::vendor_model::VendorModel>,
    retries: usize,
    format: std::option::Option<super::output_format::OutputFormat>,
    debug_mode: bool,
) -> anyhow::Result<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned + schemars::JsonSchema + crate::few_shots_traits::FewShotsOutput<T>,
{
    let overall_processing_start_time = std::time::Instant::now();
    let request_id_base = chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default();
    
    // Default logging parameters
    let log_dir = std::path::Path::new(".ras/prompts_llm_typed"); // Default log directory
    // Ensure log_dir exists (best-effort, errors handled by write_log)
    if let Err(e) = std::fs::create_dir_all(log_dir) {
        println!("Warning: Failed to create log directory '{}': {}", log_dir.display(), e);
    }
    let log_file_name_prefix = "llm_typed_interaction"; // Default prefix for log files

    let output_format = format.unwrap_or(super::output_format::OutputFormat::Json);
    let current_timestamp_str = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);
    
    let bpe = super::get_bpe::get_bpe()?;

    let schema = schemars::schema_for!(T);
    let schema_json = serde_json::to_value(&schema).context("Failed to generate JSON schema value")?;
    
    let few_shots_string = T::few_shots()
        .iter()
        .map(|val| {
            match output_format {
                super::output_format::OutputFormat::Json => serde_json::to_string(val).map_err(|e| anyhow::anyhow!(e.to_string())),
                super::output_format::OutputFormat::Yaml => serde_yaml::to_string(val).map_err(|e| anyhow::anyhow!(e.to_string())),
                super::output_format::OutputFormat::TOML => {
                    match toml::to_string_pretty(val) {
                        // Assuming crate::typed::convert_toml_multiline_to_literals is available
                        Ok(toml_str) => std::result::Result::Ok(crate::typed::convert_toml_multiline_to_literals(&toml_str)),
                        Err(e) => std::result::Result::Err(anyhow::anyhow!(e.to_string())),
                    }
                }
                super::output_format::OutputFormat::XML => std::result::Result::Ok("SEE EXAMPLES".to_string()), // Placeholder
                super::output_format::OutputFormat::Tags => std::result::Result::Ok("SEE EXAMPLES".to_string()), // Placeholder
            }
            .unwrap_or_else(|_| "".to_string())
        })
        .collect::<std::vec::Vec<_>>()
        .join("\n\n");

    let format_tag = match output_format {
        super::output_format::OutputFormat::Json => "JSON",
        super::output_format::OutputFormat::Yaml => "YAML",
        super::output_format::OutputFormat::TOML => "TOML",
        super::output_format::OutputFormat::XML => "XML",
        super::output_format::OutputFormat::Tags => "LINE_FORMAT",
    };

    let schema_string = match output_format {
        super::output_format::OutputFormat::Json | super::output_format::OutputFormat::Yaml | super::output_format::OutputFormat::TOML => {
            serde_json::to_string_pretty(&schema_json).unwrap_or_default()
        }
        super::output_format::OutputFormat::XML => "<!-- XML Schema Definition Here -->".to_string(), // Placeholder
        super::output_format::OutputFormat::Tags => "# Line Format Schema Definition Here".to_string(), // Placeholder
    };

    let full_prompt = std::format!(
        "\n<{format_tag}_SCHEMA>\n{schema}\n</{format_tag}_SCHEMA>\n\n<EXAMPLES>\n{examples}\n</EXAMPLES>\n\n<TASK>\n{task}\n</TASK>\n\nPlease respond with a valid {format_tag} object only, without any additional comments, explanations, or markdown fences.",
        format_tag = format_tag,
        schema = schema_string,
        examples = few_shots_string,
        task = prompt
    );

    if debug_mode {
        println!("LLM Typed Prompt:\n{}", full_prompt);
    }
    let input_tokens_count = bpe.encode_with_special_tokens(&full_prompt).len() as u32;

    let compiled_schema = jsonschema::JSONSchema::compile(&schema_json)
        .map_err(|e| anyhow::anyhow!("Failed to compile JSON schema: {}", e))?;

    let mut last_overall_error: std::option::Option<anyhow::Error> = None;
    let mut last_raw_response_content: std::option::Option<std::string::String> = None;

    for attempt_num in 0..=retries {
        for model_config in &models {
            let model_name_str = std::format!("{:?}", model_config); // For logging
            let attempt_request_id = std::option::Option::Some(std::format!("req_{}_{}_{}", request_id_base, model_name_str, attempt_num));
            let attempt_processing_start_time = std::time::Instant::now();

            // Construct request_payload_for_log for this specific attempt
            // This is a simplified representation. Actual vendor request bodies are more complex.
            let mut request_details_map = serde_json::Map::new();
            request_details_map.insert("prompt_text".to_string(), serde_json::Value::String(full_prompt.clone()));
            request_details_map.insert("model_name".to_string(), serde_json::Value::String(model_name_str.clone()));
            request_details_map.insert("output_format_requested".to_string(), serde_json::Value::String(format_tag.to_string()));
            
            let current_request_payload_for_log = serde_json::Value::Object(request_details_map);

            let raw_response_result = match &model_config {
                super::vendor_model::VendorModel::OpenAI(openai_model_enum) => {
                    let messages = std::vec![crate::vendors::openai::message::Message {
                        content: Some(full_prompt.clone()),
                        role: crate::vendors::openai::role::Role::User,
                        name: None,
                    }];
                    let request = crate::vendors::openai::openai_chat_completion_request::OpenAIChatCompletionRequest {
                        messages,
                        model: openai_model_enum.clone(),
                        response_format: if output_format == super::output_format::OutputFormat::Json {
                            Some(crate::vendors::openai::response_format::ResponseFormat{typ: ResponseType::JSON })
                        } else { None },
                        ..Default::default()
                    };
                    let request_body = serde_json::to_value(&request)
                        .map_err(|e| anyhow::anyhow!("Failed to serialize OpenAI request body: {}", e))?;
                    
                    // Update request_payload_for_log with actual request body
                    // let current_request_payload_for_log = request_body.clone(); 

                    crate::vendors::openai::call_gpt_with_body::call_gpt_with_body(request_body)
                        .await
                        .context(std::format!("OpenAI call failed for model {:?} (attempt {})", openai_model_enum, attempt_num + 1))
                }
                super::vendor_model::VendorModel::Gemini(gemini_model_enum) => {
                    let temperature = 0.7; // Example, could be configurable
                   crate::vendors::gemini::completion::generate_gemini_response(&full_prompt, temperature, gemini_model_enum.clone(), None)
                       .await
                       .map_err(|e| anyhow::anyhow!(e.to_string()))
                       .and_then(|output| {
                           match output {
                               crate::vendors::gemini::gemini_output::GeminiOutput::Text(s) => std::result::Result::Ok(s),
                               crate::vendors::gemini::gemini_output::GeminiOutput::Mixed{text: s, ..} => std::result::Result::Ok(s),
                               crate::vendors::gemini::gemini_output::GeminiOutput::FunctionCall(fc) => {
                                   // Serialize FunctionCallResponse to JSON string.
                                   // This ensures a string is returned, as expected by the match arm type.
                                   // Downstream parsing might still fail if T doesn't expect this structure.
                                   serde_json::to_string(&fc)
                                       .map_err(|e| anyhow::anyhow!("Failed to serialize Gemini FunctionCallResponse: {}", e))
                               }
                               crate::vendors::gemini::gemini_output::GeminiOutput::Image(inline_data) => {
                                   // For image data, return the base64 data as a string
                                   std::result::Result::Ok(inline_data.data)
                               }
                           }
                       })
                       .context(std::format!("Gemini call failed for model {:?} (attempt {})", gemini_model_enum, attempt_num + 1))
               }
               super::vendor_model::VendorModel::Claude(claude_model_enum) => {
                    let messages = std::vec![crate::vendors::claude::message::Message {
                        role: "user".to_string(),
                        content: std::vec![crate::vendors::claude::content_block::ContentBlock::Text { text: full_prompt.clone() }],
                    }];
                    let request = crate::vendors::claude::claude_message_request::ClaudeMessageRequest {
                        model: claude_model_enum.clone(),
                        messages,
                        max_tokens: 4096,
                        system: None,
                        temperature: None, top_p: None, top_k: None, stop_sequences: None, stream: None,
                    };
                    let request_body = serde_json::to_value(&request)
                        .map_err(|e| anyhow::anyhow!("Failed to serialize Claude request body: {}", e))?;
                     // let current_request_payload_for_log = request_body.clone();
                    crate::vendors::claude::call_claude_api::call_claude_api(request_body)
                        .await
                        .context(std::format!("Claude call failed for model {:?} (attempt {})", claude_model_enum, attempt_num + 1))
                }
                super::vendor_model::VendorModel::Replicate(replicate_model_enum) => {
                    crate::vendors::replicate::call_replicate_api(&full_prompt, replicate_model_enum)
                        .await
                        .context(std::format!("Replicate call failed for model {:?} (attempt {})", replicate_model_enum, attempt_num + 1))
                }
            };

            match raw_response_result {
                Ok(current_raw_response) => {
                    if debug_mode {
                        println!("Raw Response from {}:\n{}", model_name_str, current_raw_response);
                    }
                    last_raw_response_content = Some(current_raw_response.clone());
                    let output_tokens_raw_count = bpe.encode_with_special_tokens(&current_raw_response).len() as u32;
                    let total_tokens_reported = Some(input_tokens_count + output_tokens_raw_count);

                    let processing_result = match output_format {
                        super::output_format::OutputFormat::Json => {
                            super::handle_raw_json_response::handle_raw_json_response::<T>(
                                &current_raw_response, &compiled_schema,
                                attempt_request_id.clone(), current_timestamp_str.clone(), model_name_str.clone(),
                                Some(input_tokens_count), Some(output_tokens_raw_count), total_tokens_reported,
                                &current_request_payload_for_log, log_dir, log_file_name_prefix, attempt_processing_start_time,
                            )
                        }
                        super::output_format::OutputFormat::Yaml => {
                            super::handle_raw_yaml_response::handle_raw_yaml_response::<T>(
                                &current_raw_response, &compiled_schema,
                                attempt_request_id.clone(), current_timestamp_str.clone(), model_name_str.clone(),
                                Some(input_tokens_count), Some(output_tokens_raw_count), total_tokens_reported,
                                &current_request_payload_for_log, log_dir, log_file_name_prefix, attempt_processing_start_time,
                            )
                        }
                        super::output_format::OutputFormat::TOML => {
                            super::handle_raw_toml_response::handle_raw_toml_response::<T>(
                                &current_raw_response, &compiled_schema,
                                attempt_request_id.clone(), current_timestamp_str.clone(), model_name_str.clone(),
                                Some(input_tokens_count), Some(output_tokens_raw_count), total_tokens_reported,
                                &current_request_payload_for_log, log_dir, log_file_name_prefix, attempt_processing_start_time,
                            )
                        }
                        super::output_format::OutputFormat::XML | super::output_format::OutputFormat::Tags => {
                            // Logging for unsupported format attempt
                            let error_message = std::format!("Unsupported output format '{:?}' for structured parsing.", output_format);
                             let log_entry = super::llm_typed_log::LlmTypedLog {
                                request_id: attempt_request_id.clone(),
                                timestamp: current_timestamp_str.clone(),
                                model_name: model_name_str.clone(),
                                prompt_tokens: Some(input_tokens_count),
                                completion_tokens: Some(output_tokens_raw_count),
                                total_tokens: total_tokens_reported,
                                request_payload: current_request_payload_for_log.clone(),
                                response_payload: serde_json::Value::String(current_raw_response.clone()),
                                error_message: Some(error_message.clone()),
                                duration_ms: Some(attempt_processing_start_time.elapsed().as_millis() as u64),
                            };
                            let sanitized_timestamp = log_entry.timestamp.replace([':', '.'], "-").replace('+', "ZPLUS");
                            let sanitized_request_id = attempt_request_id.as_deref().unwrap_or("unknown").replace(['/', '\\', ':', '*','?', '"', '<', '>', '|'], "_");
                            let log_file_name = std::format!("{}_{}_{}.yaml",log_file_name_prefix, sanitized_timestamp, sanitized_request_id);
                            if let Err(e) = super::write_log::write_log(&log_entry, log_dir, &log_file_name) {
                                println!("Warning: Failed to write log for unsupported format: {}", e);
                            }
                            std::result::Result::Err(anyhow::anyhow!(error_message))
                        }
                    };
                    
                    match processing_result {
                        Ok(value) => return Ok(value),
                        Err(e) => {
                            last_overall_error = Some(e.context(std::format!("Processing failed for model {}", model_name_str)));
                            // Error already logged by handler, continue to next model/retry
                        }
                    }
                }
                Err(e) => {
                    last_overall_error = Some(e);
                    // Log this attempt's failure if needed, but handlers also log.
                    // For now, just store the error and continue.
                }
            }
            // If it's not the last attempt for this model, sleep before retrying the same model.
            // This sleep is removed as we iterate models first, then retries.
        }
        // If it's not the last overall retry attempt, sleep.
        if attempt_num < retries {
            tokio::time::sleep(tokio::time::Duration::from_millis(500 * (attempt_num as u64 + 1))).await;
        }
    }

    // All models and retries failed. Log final error.
    let final_error_message_str = match &last_overall_error {
        Some(err) => std::format!("{}", err),
        None => "Unknown error after all models and retries.".to_string(),
    };

    let final_log_entry = super::llm_typed_log::LlmTypedLog {
        request_id: std::option::Option::Some(std::format!("req_{}_final_error", request_id_base)),
        timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
        model_name: "all_models_failed_or_processing_error".to_string(),
        prompt_tokens: Some(input_tokens_count),
        completion_tokens: last_raw_response_content.as_ref().map(|s| bpe.encode_with_special_tokens(s).len() as u32),
        total_tokens: last_raw_response_content.as_ref().map(|s| input_tokens_count + bpe.encode_with_special_tokens(s).len() as u32),
        request_payload: serde_json::json!({ "full_prompt": full_prompt }),
        response_payload: serde_json::json!(last_raw_response_content),
        error_message: Some(final_error_message_str.clone()),
        duration_ms: Some(overall_processing_start_time.elapsed().as_millis() as u64),
    };
    
    let sanitized_timestamp = final_log_entry.timestamp.replace([':', '.'], "-").replace('+', "ZPLUS");
    let sanitized_request_id = final_log_entry.request_id.as_deref().unwrap_or("unknown_final_error").replace(['/', '\\', ':', '*','?', '"', '<', '>', '|'], "_");

    let final_log_file_name = std::format!("{}_{}_{}.yaml", log_file_name_prefix, sanitized_timestamp, sanitized_request_id);
    if let Err(e) = super::write_log::write_log(&final_log_entry, log_dir, &final_log_file_name) {
        println!("Warning: Failed to write final error log: {}", e);
    }

    std::result::Result::Err(last_overall_error.unwrap_or_else(|| {
        anyhow::anyhow!("All models failed after {} retries for prompt: '{}'", retries, prompt)
    }))
}
