//! Extracts text from files using the Gemini File API.
//!
//! This function handles file processing through the Gemini File API, which is more
//! efficient for large files like videos. It uploads the file, processes it with
//! video-optimized prompts focusing on audio transcription, and returns the extracted
//! text content. Files are automatically cleaned up after processing.

use llm::vendors::gemini::{
    completion_conversation::generate_gemini_conversation_response,
    content::Content,
    file_api_client::FileApiClient,
    file_data::FileData,
    gemini_model::GeminiModel,
    gemini_output::GeminiOutput,
    part::Part,
};

/// Extracts text from files using the Gemini File API.
///
/// This function uploads the file to the Gemini File API, processes it with
/// appropriate prompts for the file type, and returns the extracted content.
/// Optimized for video files with emphasis on audio transcription.
///
/// # Arguments
/// * `file_content` - The file content as bytes
/// * `mime_type` - The MIME type of the file
/// * `file_name` - The name of the file for display purposes
///
/// # Returns
/// A `Result` containing the extracted text or an error message
pub async fn extract_text_with_file_api(
    file_content: &[u8],
    mime_type: &str,
    file_name: &str,
) -> std::result::Result<std::string::String, std::string::String> {
    // Get API key
    let api_key = match std::env::var("GEMINI_API_KEY") {
        Ok(key) => key,
        Err(_) => return std::result::Result::Err("GEMINI_API_KEY environment variable not set".to_string()),
    };
    
    // Create File API client
    let file_client = match FileApiClient::new(api_key) {
        Ok(client) => client,
        Err(e) => return std::result::Result::Err(format!("Failed to create File API client: {e}")),
    };
    
    // Upload file to Gemini File API
    let display_name = format!("Content Extraction: {file_name}");
    let file_info = match file_client.upload_file(file_content, mime_type, &display_name).await {
        Ok(info) => info,
        Err(e) => return std::result::Result::Err(format!("Failed to upload file to Gemini API: {e}")),
    };
    
    // Create appropriate prompt based on file type
    let prompt = if mime_type.starts_with("video/") {
        r#"Extract all text content from this video. Respond with only the following three sections:

**1. AUDIO_TRANSCRIPT:**
[Complete transcription of all spoken content]

**2. VISUAL_TEXT:**
[Any text visible in the video - signs, captions, documents, etc.]

**3. CONTENT_SUMMARY:**
[Brief summary of the video's main topics and themes]

Do not include any conversational preambles or explanatory text. Start directly with the sections."#
    } else {
        // Fallback prompt for other file types that might use File API in the future
        "Extract all raw text content from the document. Store it under \"RAW_TEXT\". 
If it contains images, in addition to extracting text, prepare descriptions under \"IMAGE_DESCRIPTION\"."
    };

    // Create content with separate parts for text and file_data
    let contents = std::vec![Content {
        parts: std::vec![
            Part {
                text: Some(prompt.to_string()),
                inline_data: None,
                file_data: None,
                function_response: None,
                function_call: None,
            },
            Part {
                text: None,
                inline_data: None,
                file_data: Some(FileData {
                    mime_type: file_info.mime_type.clone(),
                    file_uri: file_info.uri.clone(),
                    display_name: None, // Don't include display_name in generateContent requests
                }),
                function_response: None,
                function_call: None,
            },
        ],
        role: None,
    }];

    // Use Gemini 2.0 Flash which has stable File API support
    let model = GeminiModel::Gemini20Flash;
    let temperature = 0.3; // Lower temperature for better transcription accuracy
    let retries = 3;

    // Process with retries
    for attempt in 0..=retries {
        match generate_gemini_conversation_response(
            contents.clone(),
            temperature,
            model.clone(),
            None,
            None,
        ).await {
            Ok(output) => match output {
                GeminiOutput::Text(text) => {
                    // Clean up: delete the file from Gemini (optional, files auto-delete after 48h)
                    if let Err(e) = file_client.delete_file(&file_info.name).await {
                        tracing::warn!("Failed to delete file {} from Gemini API: {}", file_info.name, e);
                    }
                    return std::result::Result::Ok(text);
                }
                GeminiOutput::FunctionCall(_) => {
                    // This shouldn't happen for text extraction
                    continue;
                }
                GeminiOutput::Mixed { text, .. } => {
                    // Clean up: delete the file from Gemini (optional, files auto-delete after 48h)
                    if let Err(e) = file_client.delete_file(&file_info.name).await {
                        tracing::warn!("Failed to delete file {} from Gemini API: {}", file_info.name, e);
                    }
                    return std::result::Result::Ok(text);
                }
                GeminiOutput::Image(_) => {
                    // This shouldn't happen for text extraction
                    continue;
                }
            },
            Err(e) => {
                if attempt == retries {
                    // Clean up on final failure
                    if let Err(cleanup_err) = file_client.delete_file(&file_info.name).await {
                        tracing::warn!("Failed to delete file {} from Gemini API during cleanup: {}", file_info.name, cleanup_err);
                    }
                    return std::result::Result::Err(format!(
                        "Failed to extract text using File API after {} retries: {}",
                        retries + 1,
                        e
                    ));
                }
                tracing::warn!("File API processing attempt {} failed for file {}: {}", attempt + 1, file_name, e);
            }
        }
    }

    std::result::Result::Err("Failed to extract text using File API after all retries".to_string())
}

 