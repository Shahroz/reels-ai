use crate::llm::vendors::gemini::generate_gemini_response;
use crate::llm::vendors::openai::{call_gpt_with_body, OpenAIChatCompletionRequest};
use crate::llm::vendors::openai::{Message, Role};
use anyhow::{anyhow, Context, Result};
use jsonschema::JSONSchema;
use schemars::schema_for;
use serde_json;
use serde_json::Value;
use tokio::time::{sleep, Duration};
use tracing::instrument;

// Additional imports
use crate::llm::unified::ImageData;

// Updated llm function with optional images parameter
#[instrument(skip(prompt, models, images))]
pub async fn llm(
    prompt: &str,
    models: Vec<VendorModel>,
    retries: usize,
    images: Option<Vec<ImageData>>
) -> Result<String> {
    let bpe = get_bpe();
    let _input_tokens = match &bpe {
      Ok(b) => b.encode_with_special_tokens(prompt).len(),
      Err(_) => 0,
    };
    if let Err(e) = bpe {
        eprintln!("Warning: Failed to load BPE: {}", e);
    }
    let mut last_error: Option<anyhow::Error> = None;
    for model in models {
        // If images are provided and model is OpenAI, return error as images are not supported
        if images.is_some() {
            if let VendorModel::OpenAI(_) = model {
                return Err(anyhow!("Image input is not supported for OpenAI models."));
            }
        }
        for attempt in 0..=retries {
            let result = match &model {
                VendorModel::OpenAI(openai_model) => {
                    // This branch will not be reached if images are provided
                    let request = OpenAIChatCompletionRequest::new_from_user_message(
                        prompt.to_string(),
                        openai_model.clone(),
                    );
                    let request_body = match serde_json::to_value(&request) {
                        Ok(body) => body,
                        Err(e) => {
                            last_error = Some(anyhow!("Failed to serialize OpenAI request body: {}", e));
                            continue;
                        }
                    };
                    call_gpt_with_body(request_body)
                        .await
                        .context(format!("OpenAI call failed for model {:?} (attempt {})", openai_model, attempt + 1))
                },
                VendorModel::Gemini(gemini_model) => {
                    let temperature = 0.7;
                    generate_gemini_response(prompt, temperature, gemini_model.clone(), false, images.clone())
                        .await
                        .map_err(|e| anyhow!(e.to_string()))
                        .context(format!("Gemini call failed for model {:?} (attempt {})", gemini_model, attempt + 1))
                }
            };
            match result {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < retries {
                        sleep(Duration::from_millis(500 * (attempt as u64 + 1))).await;
                    }
                }
            }
        }
    }
    Err(last_error.unwrap_or_else(|| anyhow!("All models failed after {} retries.", retries)))
}

// Updated llm_typed function with optional images parameter
#[instrument(skip(prompt, models, format, images))]
pub async fn llm_typed<T>(
    prompt: String,
    models: Vec<VendorModel>,
    retries: usize,
    format: Option<OutputFormat>,
    debug_mode: bool,
    images: Option<Vec<ImageData>>
) -> Result<T>
where
    T: SerdeSerializeTrait + DeserializeOwned + JsonSchema + FewShotsOutput<T>,
{
    let output_format = format.unwrap_or(OutputFormat::Json);
    let timestamp = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);
    let bpe = get_bpe()?;
    let schema = schema_for!(T);
    let schema_json = serde_json::to_value(&schema).context("Failed to generate JSON schema value")?;
    let few_shots_values: Vec<Value> = T::few_shots()
        .into_iter()
        .map(|v| serde_json::to_value(v))
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to serialize few shots")?;
    let few_shots_string = few_shots_values
        .iter()
        .map(|val| serde_json::to_string_pretty(val).unwrap_or_else(|_| "".to_string()))
        .collect::<Vec<_>>()
        .join("\n\n");
    let format_tag = match output_format {
        OutputFormat::Json => "JSON",
        OutputFormat::Yaml => "YAML",
    };
    let format_instruction = match output_format {
        OutputFormat::Json => "The response should be a valid JSON string without any additional comments or explanations.",
        OutputFormat::Yaml => "The response should be a valid YAML string without any additional comments or explanations.",
    };
    let schema_string = serde_json::to_string(&schema_json).unwrap_or_default();
    let format_description = format!(
        "\n{} Schema:<{}SCHEMA>{}</{}SCHEMA>\n\n. Output examples: <EXAMPLES>{}</EXAMPLES>. Important: don't use the content from the examples it is only for style guidelines.",
        format_tag,
        format_tag,
        schema_string,
        format_tag,
        few_shots_string
    );
    let full_prompt = format!(
        "You are an expert researcher. <FORMATDESCRIPTION>{}</FORMATDESCRIPTION>\n\n. Write the response based on the <TASK>, <FORMATDESCRIPTION> and <EXAMPLES>. The <TASK> is the most important and is the source of all the facts. The <FORMATDESCRIPTION> contains the {} schema and examples that should guide the format and style of the response. {} Important: Every fact must be present in the <TASK>, don't write fake information. Don't make up any numbers. {}",
        format_description,
        format_tag,
        format_instruction,
        prompt
    );
    if debug_mode { println!("LLM Typed Prompt:\n{}", full_prompt); }
    let input_tokens = bpe.encode_with_special_tokens(&full_prompt).len();
    let compiled_schema = JSONSchema::compile(&schema_json)
        .map_err(|e| anyhow!("Failed to compile JSON schema: {}", e))?;
    let mut last_error: Option<anyhow::Error> = None;
    let mut last_raw_response: Option<String> = None;

    // If images are provided and model is OpenAI, return an error
    for model in &models {
        if images.is_some() {
            if let VendorModel::OpenAI(_) = model {
                return Err(anyhow!("Image input is not supported for OpenAI models in llm_typed."));
            }
        }

    for model in models {
        for attempt in 0..=retries {
            let raw_response_result = match &model {
                VendorModel::OpenAI(openai_model) => {
                    let messages = vec![Message {
                        content: Some(full_prompt.clone()),
                        role: Role::User,
                        name: None,
                    }];
                    let request = OpenAIChatCompletionRequest {
                        messages,
                        model: openai_model.clone(),
                        ..Default::default()
                    };
                    let request_body = match serde_json::to_value(&request) {
                        Ok(body) => body,
                        Err(e) => {
                            last_error = Some(anyhow!("Failed to serialize OpenAI request body: {}", e));
                            continue;
                        }
                    };
                    call_gpt_with_body(request_body).await.context(format!(
                        "OpenAI call failed for model {:?} (attempt {})",
                        openai_model,
                        attempt + 1
                    ))
                },
                VendorModel::Gemini(gemini_model) => {
                    let temperature = 0.7;
                    generate_gemini_response(&full_prompt, temperature, gemini_model.clone(), true, images.clone())
                        .await
                        .map_err(|e| anyhow!(e.to_string()))
                        .context(format!("Gemini call failed for model {:?} (attempt {})", gemini_model, attempt + 1))
                }
            };
            match raw_response_result {
                Ok(raw_response) => {
                    last_raw_response = Some(raw_response.clone());
                    let output_tokens_raw = bpe.encode_with_special_tokens(&raw_response).len();
                    match output_format {
                        OutputFormat::Json => {
                            match hacky_json_loads(&raw_response) {
                                Some(parsed_json) => {
                                    match validate_value_unified(&compiled_schema, &parsed_json) {
                                        Ok(validated_json) => {
                                            let output_tokens_validated = match serde_json::to_string(&validated_json) {
                                                Ok(s) => bpe.encode_with_special_tokens(&s).len(),
                                                Err(_) => output_tokens_raw,
                                            };
                                            match serde_json::from_value::<T>(validated_json.clone()) {
                                                Ok(result) => {
                                                    let log_entry = LlmTypedLog {
                                                        timestamp: timestamp.clone(),
                                                        prompt: prompt.clone(),
                                                        input_token_count: input_tokens,
                                                        output_token_count: Some(output_tokens_validated),
                                                        outcome: "Success".to_string(),
                                                        details: format!("Successfully deserialized JSON. Validated JSON: {}", validated_json),
                                                    };
                                                    write_log(&log_entry);
                                                    return Ok(result);
                                                },
                                                Err(e) => {
                                                    last_error = Some(anyhow!("Failed to deserialize validated JSON to type T: {}. JSON: {}", e, validated_json));
                                                    let log_entry = LlmTypedLog {
                                                        timestamp: timestamp.clone(),
                                                        prompt: prompt.clone(),
                                                        input_token_count: input_tokens,
                                                        output_token_count: Some(output_tokens_validated),
                                                        outcome: "Error".to_string(),
                                                        details: format!("JSON Deserialization failed: {}. Validated JSON: {}", e, validated_json),
                                                    };
                                                    write_log(&log_entry);
                                                }
                                            }
                                        },
                                        Err(e) => {
                                            last_error = Some(e.context(format!(
                                                "JSON validation failed. Raw: '{}', Parsed: '{}'",
                                                raw_response, parsed_json
                                            )));
                                            let log_entry = LlmTypedLog {
                                                timestamp: timestamp.clone(),
                                                prompt: prompt.clone(),
                                                input_token_count: input_tokens,
                                                output_token_count: Some(output_tokens_raw),
                                                outcome: "Error".to_string(),
                                                details: format!("JSON Validation failed: {}. Raw Response: '{}'", last_error.as_ref().unwrap(), raw_response),
                                            };
                                            write_log(&log_entry);
                                        }
                                    }
                                },
                                None => {
                                    last_error = Some(anyhow!("Failed to parse LLM response using hacky_json_loads. Raw response: '{}'", raw_response));
                                    let log_entry = LlmTypedLog {
                                        timestamp: timestamp.clone(),
                                        prompt: prompt.clone(),
                                        input_token_count: input_tokens,
                                        output_token_count: Some(output_tokens_raw),
                                        outcome: "Error".to_string(),
                                        details: format!("JSON Parsing failed. Raw Response: '{}'", raw_response),
                                    };
                                    write_log(&log_entry);
                                }
                            }
                        },
                        OutputFormat::Yaml => {
                            match serde_yaml::from_str::<T>(&raw_response) {
                                Ok(result) => {
                                    let log_entry = LlmTypedLog {
                                        timestamp: timestamp.clone(),
                                        prompt: prompt.clone(),
                                        input_token_count: input_tokens,
                                        output_token_count: Some(output_tokens_raw),
                                        outcome: "Success".to_string(),
                                        details: "Successfully deserialized YAML.".to_string(),
                                    };
                                    write_log(&log_entry);
                                    return Ok(result);
                                },
                                Err(e) => {
                                    last_error = Some(anyhow!("Failed to deserialize YAML response: {}. Raw: '{}'", e, raw_response));
                                    let log_entry = LlmTypedLog {
                                        timestamp: timestamp.clone(),
                                        prompt: prompt.clone(),
                                        input_token_count: input_tokens,
                                        output_token_count: Some(output_tokens_raw),
                                        outcome: "Error".to_string(),
                                        details: format!("YAML Deserialization failed: {}. Raw Response: '{}'", e, raw_response),
                                    };
                                    write_log(&log_entry);
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    last_raw_response = None;
                    last_error = Some(e);
                    if attempt < retries {
                        sleep(Duration::from_millis(500 * (attempt as u64 + 1))).await;
                    }
                }
            }
        }
    }
    let final_error_details = match last_error {
        Some(ref err) => format!("{}", err),
        None => "Unknown error after all retries".to_string(),
    };
    let final_output_tokens = last_raw_response.as_ref().map(|resp| bpe.encode_with_special_tokens(resp).len());
    let log_entry = LlmTypedLog {
        timestamp: timestamp.clone(),
        prompt: prompt.clone(),
        input_token_count: input_tokens,
        output_token_count: final_output_tokens,
        outcome: "Error".to_string(),
        details: final_error_details.clone(),
    };
    write_log(&log_entry);
    Err(last_error.unwrap_or_else(|| anyhow!("All models failed after {} retries for prompt: '{}'", retries, prompt)))
}

// (Existing helper functions below remain unchanged)

// Helper function to validate JSON value against a compiled schema.
fn validate_value_unified(compiled_schema: &JSONSchema, response_value: &Value) -> Result<Value> {
    match compiled_schema.validate(response_value) {
        Ok(_) => Ok(response_value.clone()),
        Err(validation_errors) => {
            if response_value.is_array() {
                if let Some(first_element) = response_value.as_array().and_then(|arr| arr.first()) {
                    if compiled_schema.validate(first_element).is_ok() {
                        return Ok(first_element.clone());
                    }
                }
            }
            let error_messages: Vec<String> = validation_errors.map(|e| format!("{}", e)).collect();
            Err(anyhow!("Response JSON does not conform to the provided schema. Errors: [{}]. Value: {}", error_messages.join(", "), response_value))
        }
    }
}

fn hacky_json_loads(s: &str) -> Option<serde_json::Value> {
    serde_json::from_str(s).ok()
}
