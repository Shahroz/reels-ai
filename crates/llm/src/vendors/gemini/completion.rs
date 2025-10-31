use reqwest::Client;
use reqwest::StatusCode;
use std::cmp;
use std::error::Error;
use anyhow::Context;
use tokio::time::{sleep, Duration};
use crate::vendors::gemini::gcp_auth::get_gcp_authn_token;
use crate::vendors::gemini::role::Role;

// Request structures
// Response structures
/// Asynchronously sends a prompt with the specified temperature setting and returns the generated response text.
/// Grounded search is currently disabled internally.
pub async fn generate_gemini_response(
    prompt: &str,
    temperature: f64,
    model: crate::vendors::gemini::gemini_model::GeminiModel,
    tools: Option<Vec<crate::vendors::gemini::tool::Tool>>,
) -> Result<crate::vendors::gemini::gemini_output::GeminiOutput, Box<dyn Error>> {
    // Note: Imports are added at the top of the file by prepend actions.

    // Build the request object with the prompt and settings.
    let request_body = crate::vendors::gemini::gemini_request::GeminiRequest {
        contents: vec![crate::vendors::gemini::content::Content {
            role: Some(Role::User), // Single prompt, no specific role
            parts: vec![crate::vendors::gemini::part::Part {
                text: Some(prompt.to_string()),
                inline_data: None,
                file_data: None,
                function_response: None,
                function_call: None,
            }],
        }],
        generation_config: crate::vendors::gemini::generation_config::GenerationConfig {
            // response_modalities: vec!["TEXT".to_string()],
            temperature,
            max_output_tokens: model.max_tokens(),
            top_p: 1.0,
            seed: 0,
        },
        // safety_settings: vec![],
        tools, // Pass the always-None tools field
        system_instruction: None,
    };

    // API configuration (replace these with your actual values or configuration)
    let project_id = match std::env::var("GCP_PROJECT_ID") {
        Ok(id) => id.trim_matches('"').to_string(),
        Err(e) => {
            log::error!("GCP_PROJECT_ID not found: {}", e);
            return Err(Box::new(e));
        }
    };
    let location_id = match std::env::var("GCP_LOCATION") {
        Ok(loc) => loc.trim_matches('"').to_string(),
        Err(e) => {
            log::error!("GCP_LOCATION not found: {}", e);
            return Err(Box::new(e));
        }
    };

    // Create an HTTP client.
    let client = Client::builder()
        .timeout(Duration::from_secs(crate::constants::TIMEOUT))
        .build()
        .map_err(|e| {
            log::error!("Failed to build HTTP client: {}", e);
            e
        })?;

    // Retrieve an access token.
    let api_key = match std::env::var("GEMINI_API_KEY") {
        Ok(key) => key.trim_matches('"').to_string(),
        Err(e) => {
            log::error!("GEMINI_API_KEY not found: {}", e);
            return Err(Box::new(e));
        }
    };

    // Send the POST request with retry logic.
    let mut last_error: Option<String> = None;
    let max_retries = 5; // Total attempts = max_retries (0..5 means 5 attempts)

    for attempt in 0..max_retries {
        // Add a fixed 1-second delay before each attempt
        sleep(Duration::from_millis(1000)).await;

        let token = get_gcp_authn_token()
            .await
            .context("Failed to get GCP token")?;

        let url = format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
            location_id, project_id, location_id, model
        );

        let request_builder = client
            .post(url)
            .bearer_auth(token)
            .json(&request_body);

        log::debug!("logremove Gemini request builder: {:?}", request_builder);

        let response_result = request_builder.send().await;
        log::debug!("logremove Gemini response result: {:?}", response_result);

        match response_result {
            Ok(response) => {
                // Got a response, check status code using error_for_status
                match response.error_for_status() {
                    Ok(successful_response) => {
                        // Status code was 2xx, attempt to deserialize
                        match successful_response.json::<crate::vendors::gemini::api_response::ApiResponse>().await {
                            Ok(api_response) => {
                                // Successfully deserialized
                                if let Some(candidate) = api_response.candidates.first() {
                                    let mut texts = vec![];
                                    let mut function_calls = vec![];

                                    for part in &candidate.content.parts {
                                        if let Some(text) = &part.text {
                                            if !text.is_empty() {
                                                texts.push(text.clone());
                                            }
                                        }
                                        if let Some(function_call) = &part.function_call {
                                            function_calls.push(function_call.clone());
                                        }
                                    }

                                    let full_text = texts.join("");

                                    if function_calls.is_empty() {
                                        if !full_text.is_empty() {
                                            // Only text
                                            return std::result::Result::Ok(crate::vendors::gemini::gemini_output::GeminiOutput::Text(full_text));
                                        }
                                    } else { // There are function calls
                                        if !full_text.is_empty() {
                                            // Mixed: text and function calls
                                            return std::result::Result::Ok(crate::vendors::gemini::gemini_output::GeminiOutput::Mixed {
                                                text: full_text,
                                                function_calls,
                                            });
                                        } else { // Only function calls, no text
                                            if function_calls.len() == 1 {
                                                return std::result::Result::Ok(crate::vendors::gemini::gemini_output::GeminiOutput::FunctionCall(
                                                    function_calls.into_iter().next().unwrap(),
                                                ));
                                            } else {
                                                // Multiple function calls, no text. Use Mixed.
                                                return std::result::Result::Ok(crate::vendors::gemini::gemini_output::GeminiOutput::Mixed {
                                                    text: std::string::String::new(),
                                                    function_calls,
                                                });
                                            }
                                        }
                                    }
                                }
                                // Deserialized OK, but expected content not found
                                let err_msg = "No valid candidate content part (text or function call) found in response".to_string();
                                log::error!("{}", err_msg);
                                last_error = Some(err_msg);
                                break; // Exit retry loop, will return this error
                            }
                            Err(e) => {
                                // JSON deserialization error
                                let err_msg = format!("Failed to deserialize response JSON: {}", e);
                                log::error!("{}", err_msg);
                                last_error = Some(err_msg);
                                break; // Exit retry loop, will return this error (don't retry bad format)
                            }
                        }
                    }
                    Err(err) => {
                        log::error!("Failed to parse Gemini output: {}", err.to_string());
                        // error_for_status failed, meaning non-2xx status code
                        if err.status() == Some(StatusCode::TOO_MANY_REQUESTS) {
                            // It's a 429 error
                            if attempt < max_retries - 1 {
                                // Check if more retries are allowed
                                // Using exponential backoff, min 1 sec
                                let wait_time_ms = cmp::max(1000, 100 * 2u64.pow(attempt as u32));
                                sleep(Duration::from_millis(wait_time_ms)).await;
                                let warn_msg = format!("Attempt {}: Rate limited (429). Retrying after {}ms. Error: {}", attempt + 1, wait_time_ms, err);
                                log::warn!("{}", warn_msg);
                                last_error = Some(warn_msg); // Store the error message before retrying
                                continue; // Go to the next attempt
                            }
                            // Max retries reached for 429
                            let err_msg = format!("Attempt {}: Rate limited (429). Max retries reached. Error: {}", attempt + 1, err);
                            log::error!("{}", err_msg);
                            last_error = Some(err_msg);
                            break; // Exit retry loop
                        }
                        // Other non-2xx status code (e.g., 500, 403)
                        let err_msg = format!("Attempt {}: HTTP error: {}", attempt + 1, err);
                        log::error!("{}", err_msg);
                        last_error = Some(err_msg);
                        break; // Exit retry loop, don't retry these errors
                    }
                }
            }
            Err(err) => {
                log::error!("Gemini vertex error: {}", err.to_string());
                // send() failed - network level error (timeout, DNS etc.)
                if attempt < max_retries - 1 {
                    // Check if more retries are allowed
                    // Retry network errors with exponential backoff, min 1 sec
                    let wait_time_ms = cmp::max(1000, 200 * 2u64.pow(attempt as u32));
                    sleep(Duration::from_millis(wait_time_ms)).await;
                    let warn_msg = format!("Attempt {}: Network error. Retrying after {}ms. Error: {}", attempt + 1, wait_time_ms, err);
                    log::warn!("{}", warn_msg);
                    last_error = Some(warn_msg); // Store the error message before retrying
                    continue; // Go to the next attempt
                }
                // Max retries reached for network errors
                let err_msg = format!("Attempt {}: Network error. Max retries reached. Error: {}", attempt + 1, err);
                log::error!("{}", err_msg);
                last_error = Some(err_msg);
                break; // Exit retry loop
            }
        }
    }

    // If the loop finished without returning Ok, return the last recorded error
    // or a generic message if no error was somehow recorded.
    Err(Box::from(last_error.unwrap_or_else(|| {
        let msg = "Request failed after multiple retries with no specific error recorded".to_string();
        log::error!("{}", msg);
        msg
    })))
}


#[cfg(test)]
mod tests {
    //! Tests for the Gemini completion generation function.

    // Note: No 'use' statements allowed per guidelines.
    // Function under test is called via super::generate_gemini_response
    // Other items use fully qualified paths.

    use crate::vendors::gemini::gemini_output::GeminiOutput;

    #[tokio::test]
    async fn test_simple_addition_query() {
        //! Test a basic query to ensure the API call works and returns expected content.
        //! Requires GEMINI_API_KEY environment variable to be set, typically via a .env file.

        // Load .env file if present. Ignore error if not found.
        dotenvy::dotenv().ok();

        let prompt = "what is 2+2?";
        let temperature = 0.1; // Low temperature for deterministic response
        // Use fully qualified path for the model enum
        let model = crate::vendors::gemini::gemini_model::GeminiModel::Gemini25Flash; // Use a fast model

        let result = super::generate_gemini_response(prompt, temperature, model, None).await;

        match result {
            std::result::Result::Ok(output) => match output {
                crate::vendors::gemini::gemini_output::GeminiOutput::Text(response_text) => {
                    // Basic assertion: Check if the response contains the number 4.
                    // Gemini might respond conversationally, e.g., "2 + 2 equals 4."
                    println!("Gemini Text Response: {}", response_text); // Print response for debugging
                    // Use fully qualified path for assert! macro (though often in prelude)
                    std::assert!(
                        response_text.contains('4'),
                        "Response did not contain the expected digit '4'. Response: {}",
                        response_text
                    );
                }
                crate::vendors::gemini::gemini_output::GeminiOutput::FunctionCall(fc) => {
                    std::panic!("Expected text response for simple query, but got function call: {:?}", fc);
                }
                GeminiOutput::Mixed { text: response_text, .. } => {
                    // Basic assertion: Check if the response contains the number 4.
                    // Gemini might respond conversationally, e.g., "2 + 2 equals 4."
                    println!("Gemini Text Response: {}", response_text); // Print response for debugging
                    // Use fully qualified path for assert! macro (though often in prelude)
                    std::assert!(
                        response_text.contains('4'),
                        "Response did not contain the expected digit '4'. Response: {}",
                        response_text
                    );
                }
            }
            std::result::Result::Err(e) => {
                // Fail the test if the API call returns an error.
                // Use fully qualified path for panic! macro (though often in prelude)
                std::panic!("API call failed: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_function_calling_calculator() {
        //! Test function calling with a simple calculator tool.
        //! Requires GEMINI_API_KEY.

        dotenvy::dotenv().ok();

        // Define the tool parameters schema using serde_json::Value
        let params_json = serde_json::json!({
            "type": "OBJECT", // Based on the previous schema_type
            "properties": {
                "a": { "type": "integer", "description": "The first number." },
                "b": { "type": "integer", "description": "The second number." }
            },
            "required": ["a", "b"]
        });

        let calculator_tool_decl = crate::vendors::gemini::function_declaration::FunctionDeclaration {
            name: std::string::String::from("calculator_add"),
            description: std::string::String::from("Adds two integer numbers and returns the sum."),
            parameters: params_json, // Assign the serde_json::Value
        };

        let tools = Some(vec![
            crate::vendors::gemini::tool::Tool::FunctionDeclarations(
                crate::vendors::gemini::tool::FunctionDeclarationsToolWrapper {
                    function_declarations: vec![calculator_tool_decl],
                },
            ),
        ]);

        let prompt = "What is the sum of 123 and 456 using the calculator?"; // Prompt designed to trigger the tool
        let temperature = 0.1;
        let model = crate::vendors::gemini::gemini_model::GeminiModel::Gemini20Flash;

        let result = super::generate_gemini_response(prompt, temperature, model, tools).await;

        println!("Function calling test result: {:?}", result); // Log for CI or manual inspection
        assert!(result.is_ok(), "API call for function calling test failed: {:?}", result.err());
        match result.unwrap() {
            crate::vendors::gemini::gemini_output::GeminiOutput::FunctionCall(function_call) => {
                assert_eq!(function_call.name, "calculator_add");
                assert_eq!(
                    function_call.args.get("a").and_then(|v| v.as_i64()),
                    Some(123)
                );
                assert_eq!(
                    function_call.args.get("b").and_then(|v| v.as_i64()),
                    Some(456)
                );
            }
            crate::vendors::gemini::gemini_output::GeminiOutput::Text(text) => {
                panic!("Expected function call, but got text response: {}", text);
            }
            GeminiOutput::Mixed { text: text, .. } => {
                panic!("Expected function call, but got text response: {}", text);
            }
        }
    }
}
