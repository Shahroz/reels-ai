//! Extracts text from various file types.
//!
//! This function takes a byte slice representing the file content and its MIME type.
//! It attempts to parse common file formats like text, docx, and xlsx directly.
//! If the format is not directly supported (e.g., PDF, images), it falls back to
//! a multimodal LLM for content extraction.

//use crate::services::unfurl::unfurl_service::UnfurlResponse;
use calamine::{Reader, Xlsx, open_workbook_from_rs};
use docx_rust::DocxFile;
use docx_rust::document::BodyContent;
use llm::vendors::gemini::{
    completion_conversation::generate_gemini_conversation_response, content::Content,
    gemini_model::GeminiModel, gemini_output::GeminiOutput, inline_data::InlineData, part::Part,
};
use base64::{engine::general_purpose, Engine as _};

/// Extracts text from the given file content.
///
/// # Arguments
///
/// * `file_content` - A byte slice of the file's content.
/// * `mime_type` - The MIME type of the file.
///
/// # Returns
///
/// A `Result` containing the extracted text as a `String`, or an error `String`.
pub async fn extract_text(
    file_content: &[u8],
    mime_type: &str,
    file_name: &str,
) -> std::result::Result<std::string::String, std::string::String> {
    // Route videos through File API for better transcription and large file support
    if crate::services::content_extraction::should_use_file_api::should_use_file_api(mime_type) {
        return crate::services::content_extraction::extract_text_with_file_api::extract_text_with_file_api(
            file_content,
            mime_type,
            file_name,
        ).await;
    }
    // Handle plain text
    if mime_type.starts_with("text/") {
        if let Ok(text) = std::string::String::from_utf8(file_content.to_vec()) {
            return std::result::Result::Ok(text);
        }
    }

    // Handle Word documents (.docx)
    if mime_type == "application/vnd.openxmlformats-officedocument.wordprocessingml.document" {
        let cursor = std::io::Cursor::new(file_content);
        if let Ok(file) = DocxFile::from_reader(cursor) {
            if let Ok(docx) = file.parse() {
                let mut text = String::new();
                for content in docx.document.body.content {
                    if let BodyContent::Paragraph(p) = content {
                       text.push_str(&p.text());
                       text.push('\n');
                    }
                }
                return Ok(text);
            }
        }
    }

    // Handle Excel documents (.xlsx)
    if mime_type == "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" {
        let mut text = String::new();
        let cursor = std::io::Cursor::new(file_content);
        if let Ok(mut workbook) = open_workbook_from_rs::<Xlsx<_>, _>(cursor) {
            for sheet_name in workbook.sheet_names().to_owned() {
                if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                    for row in range.rows() {
                        for cell in row {
                            text.push_str(&cell.to_string());
                            text.push('\t'); // Separate cells with a tab
                        }
                        text.push('\n'); // Separate rows with a newline
                    }
                }
            }
            return Ok(text);
        }
    }

    // Fallback to multimodal LLM for other types (PDF, images, etc.)
    let prompt = "Extract all raw text content from the document. Store it under \"RAW_TEXT\". 
If it is an image, in addition to extracting text, prepare its description under \"IMAGE_DESCRIPTION\".";
    let model = GeminiModel::Gemini25Flash;
    let retries = 3;
    let temperature = 0.7;

    let base64_content = general_purpose::STANDARD.encode(file_content);
    let contents = vec![Content {
        parts: vec![
            Part {
                text: Some(prompt.to_string()),
                inline_data: None,
                file_data: None,
                function_response: None,
                function_call: None,
            },
            Part {
                text: None,
                inline_data: Some(InlineData {
                    mime_type: mime_type.to_string(),
                    data: base64_content,
                }),
                file_data: None,
                function_response: None,
                function_call: None,
            },
        ],
        role: None,
    }];

    for attempt in 0..=retries {
        match generate_gemini_conversation_response(
            contents.clone(),
            temperature,
            model.clone(),
            None,
            None,
        )
        .await
        {
            Ok(output) => match output {
                GeminiOutput::Text(text) => return Ok(text),
                GeminiOutput::FunctionCall(_) => {
                    // This case should not be reached for a text extraction prompt.
                    // We can treat it as an error or an empty response.
                    continue;
                }
                GeminiOutput::Mixed { text, .. } => {
                    return Ok(text)
                }
                GeminiOutput::Image(_) => {
                    // This case should not be reached for a text extraction prompt.
                    // We can treat it as an error or an empty response.
                    continue;
                }
            },
            Err(e) => {
                if attempt == retries {
                    return Err(format!(
                        "Failed to extract text using LLM after {} retries: {}",
                        retries + 1,
                        e
                    ));
                }
            }
        }
    }

    Err("Failed to extract text using LLM after all retries.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to read test files
    fn read_test_file(path: &str) -> Vec<u8> {
        // Correctly resolve the path relative to the crate root
        let crate_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let file_path = std::path::Path::new(&crate_root).join(path);
        match std::fs::read(&file_path) {
            Ok(content) => content,
            Err(e) => {
                panic!("Failed to read test file '{}'. Full path: {:?}. Error: {}", path, file_path, e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Skip in CI - requires GEMINI_API_KEY and test PDF file
    async fn test_extract_text_from_pdf() {
        // To run this test:
        // 1. Place a test PDF file at `crates/narrativ/backend/tests/test_data/test.pdf`
        // 2. Ensure the `GEMINI_API_KEY` environment variable is set.
        // 3. Run `cargo test -- --ignored` from the `crates/narrativ/backend` directory.
        dotenvy::dotenv().ok();

        let file_content = read_test_file("tests/test_data/test.pdf");
        let mime_type = "application/pdf";

        let result = extract_text(&file_content, mime_type, "test.pdf").await;

        assert!(result.is_ok(), "extract_text returned an error: {:?}", result.err());

        let extracted_text = result.unwrap();
        assert!(extracted_text.contains("Test bounti 2_0"));
    }

    #[tokio::test]
    async fn test_extract_text_from_docx() {
        // To run this test:
        // 1. Create a test .docx file at `crates/narrativ/backend/tests/test_data/test.docx`
        //    containing the text "test bounti 2_0".
        dotenvy::dotenv().ok();

        let file_content = read_test_file("tests/test_data/test.docx");
        let mime_type = "application/vnd.openxmlformats-officedocument.wordprocessingml.document";

        let result = extract_text(&file_content, mime_type, "test.docx").await;

        assert!(result.is_ok(), "extract_text returned an error: {:?}", result.err());
        let extracted_text = result.unwrap();
        assert!(extracted_text.contains("Test bounti 2_0"));
    }

    #[tokio::test]
    async fn test_extract_text_from_xlsx() {
        // To run this test:
        // 1. Create a test .xlsx file at `crates/narrativ/backend/tests/test_data/test.xlsx`
        //    with "test bounti 2_0" in one of the cells.
        dotenvy::dotenv().ok();

        let file_content = read_test_file("tests/test_data/test.xlsx");
        let mime_type = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";

        let result = extract_text(&file_content, mime_type, "test.xlsx").await;

        assert!(result.is_ok(), "extract_text returned an error: {:?}", result.err());
        let extracted_text = result.unwrap();
        assert!(extracted_text.contains("Test bounti 2_0"));
    }
} 