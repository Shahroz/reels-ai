//! Handles multi-turn conversation requests with the Google Gemini API.
//!
//! This module provides functionality to send a sequence of content parts,
//! each with an optional role (e.g., "user", "model"), to the Gemini API
//! and receive a generated response. It adapts logic from `completion.rs`.
//! Uses fully qualified paths for all dependencies and adheres to coding guidelines.

/// Asynchronously sends a conversation (a list of `Content` objects) to the Gemini API
/// with the specified temperature and model, then returns the generated response text.
///
/// This function is designed for multi-turn conversations where each `Content` object
/// can have a `role` (e.g., "user", "model") and multiple `Part`s (text or inline data).
pub async fn generate_gemini_conversation_response(
    contents: std::vec::Vec<crate::vendors::gemini::content::Content>,
    temperature: f64,
    model: crate::vendors::gemini::gemini_model::GeminiModel,
    system_instruction: Option<String>,
    tools_config: Option<std::vec::Vec<crate::vendors::gemini::tool::Tool>>,
) -> std::result::Result<crate::vendors::gemini::gemini_output::GeminiOutput, std::boxed::Box<dyn std::error::Error>> {
    let system_instruction_struct = match system_instruction {
        None => None,
        Some(instruction) => {
            Some(crate::vendors::gemini::system_instruction::SystemInstruction{
                parts: crate::vendors::gemini::system_instruction_text_payload::SystemInstructionTextPayload{
                    text: instruction
                }
            })
        }
    };

    // Build the request object with the provided contents and settings.
    let request_body = crate::vendors::gemini::gemini_request::GeminiRequest {
        contents, // Direct assignment from the function argument
        generation_config: crate::vendors::gemini::generation_config::GenerationConfig {
            temperature,
            max_output_tokens: model.max_tokens(),
            top_p: 1.0,
            seed: 0,
        },
        tools: tools_config,
        system_instruction: system_instruction_struct
    };

    // Create an HTTP client.
    let client = reqwest::Client::builder()
        .timeout(tokio::time::Duration::from_secs(crate::constants::TIMEOUT))
        .build()
        .map_err(|e| {
            log::error!("Failed to build HTTP client: {}", e);
            e
        })?;

    // Retrieve the API key.
    let api_key = match std::env::var("GEMINI_API_KEY") {
        Ok(key) => key.trim_matches('"').to_string(),
        Err(e) => {
            log::error!("GEMINI_API_KEY not found: {}", e);
            return std::result::Result::Err(std::boxed::Box::new(e));
        }
    };

    // Send the POST request with retry logic.
    let mut last_error: Option<std::string::String> = None;
    let max_retries = 5;

    for attempt in 0..max_retries {
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await; // Fixed 1s delay

        let request_builder = client
            .post(format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                model.id(),
                api_key
            ))
            .json(&request_body);

        let response_result = request_builder.send().await;

        match response_result {
            Ok(response) => {
                match response.error_for_status() {
                    Ok(successful_response) => {
                        match successful_response.json::<serde_json::Value>().await {
                            Ok(response_json) => {
                                // Extract content from the first candidate
                                if let Some(candidates) = response_json.get("candidates").and_then(|c| c.as_array()) {
                                    if let Some(first_candidate) = candidates.get(0) {
                                        if let Some(content) = first_candidate.get("content") {
                                            if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
                                                // Prioritize finding an image part in multi-part responses
                                                let image_part = parts.iter().find(|part| part.get("inlineData").is_some());

                                                if let Some(part) = image_part {
                                                    if let Some(inline_data) = part.get("inlineData") {
                                                        let mime_type = inline_data.get("mimeType").and_then(|m| m.as_str()).unwrap_or("").to_string();
                                                        let data = inline_data.get("data").and_then(|d| d.as_str()).unwrap_or("").to_string();
                                                        return std::result::Result::Ok(crate::vendors::gemini::gemini_output::GeminiOutput::Image(
                                                            crate::vendors::gemini::inline_data::InlineData { mime_type, data },
                                                        ));
                                                    }
                                                }

                                                // Fallback to text or function call if no image part is found
                                                if let Some(first_part) = parts.get(0) {
                                                    if let Some(text) = first_part.get("text").and_then(|t| t.as_str()) {
                                                        return std::result::Result::Ok(crate::vendors::gemini::gemini_output::GeminiOutput::Text(text.to_string()));
                                                    } else if let Some(function_call) = first_part.get("functionCall").cloned() {
                                                        if let Ok(call) = serde_json::from_value(function_call) {
                                                            return std::result::Result::Ok(crate::vendors::gemini::gemini_output::GeminiOutput::FunctionCall(call));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                let err_msg = "No valid content found in Gemini response".to_string();
                                log::error!("{}", err_msg);
                                last_error = Some(err_msg);
                                break;
                            }
                            Err(e) => {
                                let err_msg = format!("Failed to deserialize conversation response JSON: {}", e);
                                log::error!("{}", err_msg);
                                last_error = Some(err_msg);
                                break;
                            }
                        }
                    }
                    Err(err) => {
                        if err.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
                            if attempt < max_retries - 1 {
                                let wait_time_ms = std::cmp::max(1000, 100 * 2u64.pow(attempt as u32));
                                tokio::time::sleep(tokio::time::Duration::from_millis(wait_time_ms)).await;
                                let warn_msg = format!("Attempt {}: Rate limited (429) for conversation. Retrying after {}ms. Error: {}", attempt + 1, wait_time_ms, err);
                                log::warn!("{}", warn_msg);
                                last_error = Some(warn_msg);
                                continue;
                            }
                            let err_msg = format!("Attempt {}: Rate limited (429) for conversation. Max retries reached. Error: {}", attempt + 1, err);
                            log::error!("{}", err_msg);
                            last_error = Some(err_msg);
                            break;
                        }
                        let err_msg = format!("Attempt {}: HTTP error in conversation: {}", attempt + 1, err);
                        log::error!("{}", err_msg);
                        last_error = Some(err_msg);
                        break;
                    }
                }
            }
            Err(err) => {
                if attempt < max_retries - 1 {
                    let wait_time_ms = std::cmp::max(1000, 200 * 2u64.pow(attempt as u32));
                    tokio::time::sleep(tokio::time::Duration::from_millis(wait_time_ms)).await;
                    let warn_msg = format!("Attempt {}: Network error in conversation. Retrying after {}ms. Error: {}", attempt + 1, wait_time_ms, err);
                    log::warn!("{}", warn_msg);
                    last_error = Some(warn_msg);
                    continue;
                }
                let err_msg = format!("Attempt {}: Network error in conversation. Max retries reached. Error: {}", attempt + 1, err);
                log::error!("{}", err_msg);
                last_error = Some(err_msg);
                break;
            }
        }
    }

    std::result::Result::Err(std::boxed::Box::from(last_error.unwrap_or_else(|| {
        let msg = "Conversation request failed after multiple retries with no specific error recorded".to_string();
        log::error!("{}", msg);
        msg
    })))
}

#[cfg(test)]
mod tests {
    //! Tests for the Gemini conversation generation function.

    use crate::vendors::gemini::gemini_output::GeminiOutput;

    #[tokio::test]
    async fn test_simple_conversation() {
        //! Test a basic two-turn conversation.
        //! Requires GEMINI_API_KEY environment variable.

        dotenvy::dotenv().ok();
        if std::env::var("GEMINI_API_KEY").is_err() {
            println!("Skipping test_simple_conversation: GEMINI_API_KEY not set.");
            return;
        }

        let conversation_history = std::vec![
            crate::vendors::gemini::content::Content {
                role: Some(crate::vendors::gemini::role::Role::User),
                parts: std::vec![crate::vendors::gemini::part::Part {
                    text: Some("Hello there!".to_string()),
                    inline_data: None,
                    file_data: None,
                    function_response: None,
                    function_call: None,
                }],
            },
            crate::vendors::gemini::content::Content {
                role: Some(crate::vendors::gemini::role::Role::Model),
                parts: std::vec![crate::vendors::gemini::part::Part {
                    text: Some("Hi! How can I help you today?".to_string()),
                    inline_data: None,
                    file_data: None,
                    function_response: None,
                    function_call: None,
                }],
            },
            crate::vendors::gemini::content::Content {
                role: Some(crate::vendors::gemini::role::Role::User),
                parts: std::vec![crate::vendors::gemini::part::Part {
                    text: Some("What is the capital of France?".to_string()),
                    inline_data: None,
                    file_data: None,
                    function_response: None,
                    function_call: None,
                }],
            },
        ];

        let temperature = 0.1;
        let model = crate::vendors::gemini::gemini_model::GeminiModel::Gemini20Flash;

        let result = super::generate_gemini_conversation_response(conversation_history, temperature, model, None, None).await;

        match result {
            std::result::Result::Ok(output) => match output {
                crate::vendors::gemini::gemini_output::GeminiOutput::Text(text_response) => {
                    println!("Gemini Conversation Response: {}", text_response);
                    std::assert!(
                        text_response.to_lowercase().contains("paris"),
                        "Response did not contain 'paris'. Response: {}",
                        text_response
                    );
                }
                crate::vendors::gemini::gemini_output::GeminiOutput::Mixed { text: text_response, .. } => {
                    println!("Gemini Conversation Response: {}", text_response);
                    std::assert!(
                        text_response.to_lowercase().contains("paris"),
                        "Response did not contain 'paris'. Response: {}",
                        text_response
                    );
                }
                crate::vendors::gemini::gemini_output::GeminiOutput::FunctionCall(_) => std::panic!("Expected text response, got function call"),
                crate::vendors::gemini::gemini_output::GeminiOutput::Image(_) => std::panic!("Expected text response, got image data"),
            }
            std::result::Result::Err(e) => {
                std::panic!("API call for conversation failed: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_conversation_with_image_placeholder() {
        //! Test a conversation that includes a part intended for an image (though not sending actual image data).
        //! This primarily tests the data structures.
        //! Requires GEMINI_API_KEY environment variable.

        dotenvy::dotenv().ok();
        if std::env::var("GEMINI_API_KEY").is_err() {
            println!("Skipping test_conversation_with_image_placeholder: GEMINI_API_KEY not set.");
            return;
        }

        let conversation_history = std::vec![
            crate::vendors::gemini::content::Content {
                role: Some(crate::vendors::gemini::role::Role::User),
                parts: std::vec![
                    crate::vendors::gemini::part::Part {
                        text: Some("Describe this image:".to_string()),
                        inline_data: None,
                        file_data: None,
                        function_response: None,
                        function_call: None,
                    },
                    crate::vendors::gemini::part::Part {
                        text: None,
                        inline_data: Some(crate::vendors::gemini::inline_data::InlineData {
                             mime_type: "image/png".to_string(),
                             data: "pretend_this_is_base64_image_data".to_string(),
                        }),
                        file_data: None,
                        function_response: None,
                        function_call: None,
                    },
                    crate::vendors::gemini::part::Part {
                        text: Some("What is shown?".to_string()),
                        inline_data: None,
                        file_data: None,
                        function_response: None,
                        function_call: None,
                    }
                ],
            },
        ];

        let temperature = 0.7; // Higher temp as it might be more creative with a fake image
        let model = crate::vendors::gemini::gemini_model::GeminiModel::Gemini20Flash;

        let result = super::generate_gemini_conversation_response(conversation_history, temperature, model, None, None).await;

        match result {
            std::result::Result::Ok(output) => match output {
                crate::vendors::gemini::gemini_output::GeminiOutput::Text(text_response) => {
                    println!("Gemini Image Conversation Response: {}", text_response);
                    // We don't know what it will say, but it shouldn't error out on the request structure.
                    std::assert!(!text_response.is_empty(), "Response was empty for image placeholder test.");
                }
                crate::vendors::gemini::gemini_output::GeminiOutput::FunctionCall(_) => std::panic!("Expected text response for image, got function call"),
                crate::vendors::gemini::gemini_output::GeminiOutput::Image(_) => std::panic!("Expected text response for image, got image data"),
                GeminiOutput::Mixed { text: text_response, .. } => {
                    println!("Gemini Image Conversation Response: {}", text_response);
                    // We don't know what it will say, but it shouldn't error out on the request structure.
                    std::assert!(!text_response.is_empty(), "Response was empty for image placeholder test.");
                }
            }
            std::result::Result::Err(e) => {
                // This might fail if the API strictly validates base64, but the goal is to test request formation.
                // The API might return an error if the "image data" is invalid. This is acceptable for this test.
                println!("API call for image conversation placeholder failed as potentially expected: {:?}", e);
                // For now, we'll consider an error here as a "pass" if it's about invalid input,
                // as the primary goal is testing the Rust struct serialization.
                // A more robust test would mock the API or use a tiny valid base64 image.
               // Example error: "Request payload is invalid: image decoding failed"
               let error_string = format!("{:?}", e);
               std::assert!(
                    error_string.contains("INVALID_ARGUMENT") || error_string.contains("image decoding failed") || error_string.contains("Invalid base64") || error_string.contains("400 Bad Request"),
                    "Error was not an expected invalid argument/image decoding/400 Bad Request error: {:?}", e
               );

           }
        }
    }

    #[tokio::test]
    async fn test_conversation_with_tool_use() {
        //! Tests a multi-turn conversation involving a tool call (function calling).
        //! Requires GEMINI_API_KEY.
        dotenvy::dotenv().ok();
        if std::env::var("GEMINI_API_KEY").is_err() {
            println!("Skipping test_conversation_with_tool_use: GEMINI_API_KEY not set.");
            return;
       }

       // 1. Define a function declaration for the tool
       let parameters = serde_json::json!{{
           "type": "OBJECT",
           "properties": {
               "location": {
                   "type": "string",
                   "description": "The city, e.g., 'San Francisco'"
               }
           },
           "required": ["location"]
        }};
        let function_declaration = crate::vendors::gemini::function_declaration::FunctionDeclaration {
            name: std::string::String::from("get_current_weather"),
            description: std::string::String::from("Get the current weather in a given location"),
            parameters,
        };

        let function_declarations_wrapper = crate::vendors::gemini::tool::FunctionDeclarationsToolWrapper {
            function_declarations: std::vec![function_declaration],
        };
        let tools_config = Some(std::vec![
            crate::vendors::gemini::tool::Tool::FunctionDeclarations(function_declarations_wrapper)
        ]);

        // 2. Initial user message to trigger the tool
        let mut conversation_history = std::vec![
            crate::vendors::gemini::content::Content {
                role: Some(crate::vendors::gemini::role::Role::User),
                parts: std::vec![crate::vendors::gemini::part::Part {
                    text: Some("What is the weather like in London?".to_string()),
                    inline_data: None,
                    file_data: None,
                    function_response: None,
                    function_call: None,
                }],
            },
        ];

        let temperature = 0.1;
        let model = crate::vendors::gemini::gemini_model::GeminiModel::Gemini20Flash; // Or another model that supports tools

        // 3. First API call - expect a function call request
        let result1 = super::generate_gemini_conversation_response(
            conversation_history.clone(), // Clone as we'll reuse/extend
            temperature,
            model.clone(),
            None,
            tools_config.clone(),
        ).await;

        let function_call_details = match result1 {
            Ok(crate::vendors::gemini::gemini_output::GeminiOutput::FunctionCall(call_details)) => {
                println!("Received function call request: {:?}", call_details);
                assert_eq!(call_details.name, "get_current_weather");
                // Args might be {"location": "London"} or similar
                assert!(call_details.args.as_object().unwrap().contains_key("location"));
                assert_eq!(call_details.args.as_object().unwrap().get("location").unwrap().as_str().unwrap().to_lowercase(), "london");
                call_details
            }
            Ok(crate::vendors::gemini::gemini_output::GeminiOutput::Text(text)) => {
                panic!("Expected function call, but got text: {}", text);
            }
            Ok(crate::vendors::gemini::gemini_output::GeminiOutput::Image(_)) => {
                panic!("Expected function call, but got image data");
            }
            Ok(GeminiOutput::Mixed { text: text, .. }) => {
                panic!("Expected function call, but got text: {}", text);
            }
            Err(e) => panic!("API call 1 (tool trigger) failed: {:?}", e),
        };

        // 4. Simulate tool execution and prepare the function result content
        // Add model's function call turn to history
        conversation_history.push(crate::vendors::gemini::content::Content {
            role: Some(crate::vendors::gemini::role::Role::Model),
            parts: std::vec![crate::vendors::gemini::part::Part {
                text: None,
                inline_data: None,
                file_data: None,
                function_response: None,
                function_call: Some(function_call_details.clone()),
            }],
        });

        // Simulate executing "get_current_weather" for "London" and getting "rainy"
        let tool_output_content = crate::vendors::gemini::content::Content {
            role: None,
            parts: std::vec![crate::vendors::gemini::part::Part {
                text: Some("{\"weather\": \"rainy and damp\"}".to_string()), // Result as JSON string
                inline_data: None,
                file_data: None,
                function_response: None,
                function_call: None,
            }],
        };
        let function_result_payload = crate::vendors::gemini::function_result_part::FunctionResultPart {
            name: function_call_details.name.clone(),
            response: tool_output_content,
        };
        conversation_history.push(crate::vendors::gemini::content::Content {
            role: Some(crate::vendors::gemini::role::Role::Function),
            parts: std::vec![crate::vendors::gemini::part::Part {
                text: None,
                inline_data: None,
                file_data: None,
                function_response: Some(function_result_payload),
                function_call: None,
            }],
        });

        // 5. Second API call - send function result, expect final text response
        let result2 = super::generate_gemini_conversation_response(conversation_history, temperature, model, None, tools_config).await;

        match result2 {
            Ok(crate::vendors::gemini::gemini_output::GeminiOutput::Text(final_text)) => {
                println!("Final response after tool use: {}", final_text);
                assert!(final_text.to_lowercase().contains("rainy") || final_text.to_lowercase().contains("damp"));
            }
            Ok(crate::vendors::gemini::gemini_output::GeminiOutput::FunctionCall(fc)) => panic!("Expected final text, but got another function call: {:?}", fc),
            Ok(crate::vendors::gemini::gemini_output::GeminiOutput::Image(_)) => panic!("Expected final text, but got image data"),
            Ok(GeminiOutput::Mixed { text: final_text, .. }) => {
                println!("Final response after tool use: {}", final_text);
                assert!(final_text.to_lowercase().contains("rainy") || final_text.to_lowercase().contains("damp"));
            }
            Err(e) => panic!("API call 2 (after tool result) failed: {:?}", e),
        }
    }
}
