//! Provides a unified, non-typed LLM (Large Language Model) calling function.
//!
//! This function, `llm`, attempts to call various LLM vendors with a given prompt.
//! It supports retries and iterates through a list of specified models.
//! It returns the raw string response from the first successful model call.
//! This function is primarily for non-structured text generation.

use anyhow::Context;

// Note: This function exceeds the 50 LoC guideline due to the sequential vendor dispatch and retry logic,
// which is difficult to break down further while maintaining clarity.
pub async fn llm(
    debug_prompt_enabled: bool,
    prompt: &str,
    models: std::vec::Vec<super::vendor_model::VendorModel>,
    retries: usize,
) -> anyhow::Result<std::string::String> {
    // Calculate input tokens (best effort)
   let bpe_result = super::get_bpe::get_bpe();
    let _input_tokens = match &bpe_result {
        std::result::Result::Ok(b) => b.encode_with_special_tokens(prompt).len(),
        std::result::Result::Err(_) => 0, // Default to 0 if BPE loading fails
    };
    if let std::result::Result::Err(e) = &bpe_result {
        // Using std::println! as it's in prelude
        std::println!("Warning: Failed to calculate input tokens for llm call: {}", e);
    }

    let mut last_error: Option<anyhow::Error> = None;
    if debug_prompt_enabled {
        std::println!("LLM Prompt:\n{}", prompt);
    }
    for model in models {
        for attempt in 0..=retries {
            let result: anyhow::Result<std::string::String> = match &model {
               super::vendor_model::VendorModel::OpenAI(openai_model) => {
                    let request =
                        crate::vendors::openai::openai_chat_completion_request::OpenAIChatCompletionRequest::new_from_user_message(
                            prompt.to_string(),
                            openai_model.clone(),
                        );
                    let request_body = match serde_json::to_value(&request) { // serde_json::to_value is a function
                        std::result::Result::Ok(body) => body,
                        std::result::Result::Err(e) => {
                            last_error = Some(anyhow::anyhow!( // anyhow::anyhow! is a macro
                                "Failed to serialize OpenAI request body: {}",
                                e
                            ));
                            continue;
                        }
                    };
                    crate::vendors::openai::call_gpt_with_body::call_gpt_with_body(request_body)
                        .await
                        .context(format!( // .context() comes from `use anyhow::Context;`
                            "OpenAI call failed for model {:?} (attempt {})",
                            openai_model,
                            attempt + 1
                        ))
                }
               super::vendor_model::VendorModel::Gemini(gemini_model) => {
                    let temperature = 0.7;
                    crate::vendors::gemini::completion::generate_gemini_response(
                        prompt,
                        temperature,
                        gemini_model.clone(),
                        None,
                    )
                    .await
                    .map_err(|e| anyhow::anyhow!(e.to_string())) // Convert Box<dyn Error> to anyhow::Error
                    .and_then(|gemini_output| match gemini_output {
                        crate::vendors::gemini::gemini_output::GeminiOutput::Text(text) => std::result::Result::Ok(text),
                        crate::vendors::gemini::gemini_output::GeminiOutput::Mixed{text, ..} => std::result::Result::Ok(text),
                        crate::vendors::gemini::gemini_output::GeminiOutput::FunctionCall(_) => {
                            std::result::Result::Err(anyhow::anyhow!(
                                "Gemini call for model {:?} returned a function call, but a text response was expected.",
                                gemini_model
                            ))
                        }
                        crate::vendors::gemini::gemini_output::GeminiOutput::Image(_) => {
                            std::result::Result::Err(anyhow::anyhow!(
                                "Gemini call for model {:?} returned image data, but a text response was expected.",
                                gemini_model
                            ))
                        }
                    })
                    .context(format!(
                        "Gemini call failed for model {:?} (attempt {})",
                        gemini_model,
                        attempt + 1
                    ))
                }
               super::vendor_model::VendorModel::Claude(claude_model) => {
                    let messages = std::vec![crate::vendors::claude::message::Message {
                        role: "user".to_string(),
                        content: std::vec![crate::vendors::claude::content_block::ContentBlock::Text {
                            text: prompt.to_string(),
                        }],
                    }];
                    let request =
                        crate::vendors::claude::claude_message_request::ClaudeMessageRequest {
                            model: claude_model.clone(),
                            messages,
                            max_tokens: 4096,
                            system: None,
                            temperature: None,
                            top_p: None,
                            top_k: None,
                            stop_sequences: None,
                            stream: None,
                        };
                    let request_body = match serde_json::to_value(&request) {
                        std::result::Result::Ok(body) => body,
                        std::result::Result::Err(e) => {
                            last_error = Some(anyhow::anyhow!(
                                "Failed to serialize Claude request body: {}",
                                e
                            ));
                            continue;
                        }
                    };
                    crate::vendors::claude::call_claude_api::call_claude_api(request_body)
                        .await
                        .context(format!(
                            "Claude call failed for model {:?} (attempt {})",
                            claude_model,
                            attempt + 1
                        ))
                }
               super::vendor_model::VendorModel::Replicate(
                    replicate_model,
                ) => crate::vendors::replicate::call_replicate_api(prompt, replicate_model)
                    .await
                    .context(format!(
                        "Replicate call failed for model {:?} (attempt {})",
                        replicate_model,
                        attempt + 1
                    )),
            };

            match result {
                std::result::Result::Ok(response) => {
                    if debug_prompt_enabled {
                        std::println!("Response:\n{}", response);
                    }
                    return std::result::Result::Ok(response);
                }
                std::result::Result::Err(e) => {
                    last_error = Some(e);
                    if attempt < retries {
                        // tokio::time::sleep and tokio::time::Duration are function and type, not traits
                        tokio::time::sleep(tokio::time::Duration::from_millis(
                            500 * (attempt as u64 + 1),
                        ))
                        .await;
                    }
                }
            }
        }
    }

    std::result::Result::Err(
        last_error.unwrap_or_else(|| anyhow::anyhow!("All models failed after {} retries.", retries)),
    )
}
