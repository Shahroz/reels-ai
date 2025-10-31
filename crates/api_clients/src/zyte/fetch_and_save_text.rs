//! Fetches HTML from a URL using Zyte, extracts text content, and saves it to a file.
//!
//! This module provides the `fetch_and_save_text` function which orchestrates
//! the process of using the Zyte API to get rendered HTML, parsing it to
//! extract human-readable text, and writing that text to a specified output path.
//! Adheres strictly to the one-item-per-file guideline and uses fully qualified paths.

//! Revision History
//! - 2025-04-24T07:14:48Z @AI: Initial implementation based on instruction.

/// Fetches browser HTML via Zyte, extracts text, and saves to a file.
///
/// 1. Initializes a `ZyteClient`.
/// 2. Creates a `ZyteRequest` for the URL, requesting `browserHtml`.
/// 3. Calls the Zyte API using `client.extract()`.
/// 4. Parses the returned HTML using `scraper`.
/// 5. Extracts and concatenates all text nodes.
/// 6. Writes the concatenated text to `output_path`, overwriting if it exists.
///
/// # Arguments
/// * `url` - The URL to fetch HTML from.
/// * `output_path` - The file path to save the extracted text to.
///
/// # Returns
/// `Ok(())` on success, or a `crate::zyte::error::ZyteError` on failure.
pub async fn fetch_and_save_text(url: &str, output_path: &str) -> Result<(), crate::zyte::error::ZyteError> {
    // 1. Initialize ZyteClient
    let client = crate::zyte::client::ZyteClient::new()?;

    // 2. Create ZyteRequest
    let request = crate::zyte::request::ZyteRequest {
        url: url.to_string(),
        http_response_body: None,
        browser_html: Some(true), // Request rendered HTML
        screenshot: None,
        screenshot_options: None,
        actions: None,
    };

    // 3. Call Zyte API
    let response = client.extract(&request).await?;

    // 4. Get browser_html
    let html_content = response.browser_html
        .ok_or_else(|| crate::zyte::error::ZyteError::InvalidInput("Zyte API did not return browserHtml".to_string()))?;

    if html_content.is_empty() {
        return Err(crate::zyte::error::ZyteError::InvalidInput("Received empty browserHtml from Zyte API".to_string()));
    }

    // 5. Parse HTML
    let document = ::scraper::Html::parse_document(&html_content);

    // 6. Extract and concatenate text nodes
    let mut extracted_text = ::std::string::String::new();
    for text_node in document.root_element().text() {
        let trimmed = text_node.trim();
        if !trimmed.is_empty() {
            extracted_text.push_str(trimmed);
            extracted_text.push('\n'); // Add newline between fragments for basic readability
        }
    }
    // Remove trailing newline if present
    if extracted_text.ends_with('\n') {
        extracted_text.pop();
    }


    // 7. Create/overwrite output file
    let mut file = ::tokio::fs::File::create(output_path)
        .await
        .map_err(|e| crate::zyte::error::ZyteError::IoError(format!("Failed to create/open file '{output_path}': {e}")))?;

    // 8. Write concatenated text to file
    ::tokio::io::AsyncWriteExt::write_all(&mut file, extracted_text.as_bytes())
        .await
        .map_err(|e| crate::zyte::error::ZyteError::IoError(format!("Failed to write to file '{output_path}': {e}")))?;

    // 9. Return Ok
    Ok(())
}

#[cfg(test)]
mod tests {
    // Note: Using fully qualified paths.
    // Tests involving real network calls or file system access should be #[ignore]d by default.

    use crate::fetch_and_save_text;

    #[test]
    fn test_text_extraction_logic() {
        // Test the scraper text extraction logic on a hardcoded HTML string.
        // This avoids network and file I/O.
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head><title>Test Page</title></head>
            <body>
                <h1>Header</h1>
                <p>This is paragraph 1.</p>
                <div><p> This is paragraph 2 within a div. </p></div>
                <script>console.log("Ignore me");</script>
                <style>.bold { font-weight: bold; }</style>
                <p>Final sentence. </p>
            </body>
            </html>
        "#;
        let document = ::scraper::Html::parse_document(html);
        let mut extracted_text = ::std::string::String::new();
        for text_node in document.root_element().text() {
            let trimmed = text_node.trim();
             if !trimmed.is_empty() {
                extracted_text.push_str(trimmed);
                 extracted_text.push('\n');
             }
        }
         if extracted_text.ends_with('\n') {
            extracted_text.pop();
        }


        let expected_text = "Test Page\nHeader\nThis is paragraph 1.\nThis is paragraph 2 within a div.\nFinal sentence.";
        assert_eq!(extracted_text, expected_text);
    }

    #[tokio::test]
    #[ignore] // Ignored because it requires network access (Zyte API key) and writes a file.
    async fn test_integration_fetch_and_save() {
        // This test requires a valid ZYTE_API_KEY in the environment.
        // It also creates a file in the temporary directory.
        ::dotenvy::dotenv().ok();
        if ::std::env::var("ZYTE_API_KEY").is_err() {
            println!("Skipping test_integration_fetch_and_save: ZYTE_API_KEY not set.");
            return;
        }

        let url = "http://books.toscrape.com/"; // A simple site for testing
        let mut temp_dir = ::std::env::temp_dir();
        temp_dir.push(format!("test_output_{}.txt", ::rand::random::<u64>()));
        let output_path = temp_dir.to_str().expect("Failed to create temp path string");

        let result = fetch_and_save_text(url, output_path).await;

        assert!(result.is_ok(), "fetch_and_save_text failed: {:?}", result.err());

        // Verify file content (basic check)
        let content = ::tokio::fs::read_to_string(output_path)
            .await
            .expect("Failed to read output file");
        assert!(!content.is_empty(), "Output file is empty");
        assert!(content.contains("Books to Scrape"), "Output content seems incorrect"); // Check for expected text

        // Clean up the temporary file
        ::tokio::fs::remove_file(output_path)
            .await
            .expect("Failed to remove temp output file");
    }
}
