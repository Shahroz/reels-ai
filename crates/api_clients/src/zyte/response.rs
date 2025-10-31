use base64::engine::{general_purpose::STANDARD, Engine as _};
// Represents the main response structure from the Zyte API extract endpoint.
//
// Defines the structure capturing the data returned by the Zyte API,
// including the final URL, status code, and optional fields like
// HTML content, response body, and screenshots (base64 encoded).
// Includes a helper method to decode the base64 response body.

// Revision History
// - 2025-04-15T15:13:13Z @AI: Add `decoded_http_response_body` method and tests.
// - 2025-04-15T15:06:21Z @AI: Created file based on one-item-per-file guideline.

/// Represents the main response structure from the Zyte API.
#[derive(Debug, ::serde::Deserialize)]
pub struct ZyteResponse {
    /// The final URL after redirects.
    pub url: String,
    /// The HTTP status code of the response.
    #[serde(rename = "statusCode")]
    pub status_code: u16,
    /// The raw HTTP response body, base64 encoded.
    #[serde(rename = "httpResponseBody")]
    pub http_response_body: Option<String>,
    /// The rendered HTML content from the browser.
    #[serde(rename = "browserHtml")]
    pub browser_html: Option<String>,
    /// The screenshot image, base64 encoded.
    pub screenshot: Option<String>,
    // TODO: Add other potential response fields based on API docs.
}

impl ZyteResponse {
    /// Decodes the base64 encoded `httpResponseBody` if present.
    ///
    /// Returns `Ok(Some(Vec<u8>))` containing the decoded bytes on success,
    /// `Ok(None)` if `httpResponseBody` is None, or `Err(base64::DecodeError)`
    /// if decoding fails.
    pub fn decoded_http_response_body(&self) -> Result<Option<Vec<u8>>, ::base64::DecodeError> {
        match &self.http_response_body {
            Some(encoded_body) => {
                // Use fully qualified path for decode as per guidelines
                match STANDARD.decode(encoded_body) {
                    Ok(decoded_bytes) => Result::Ok(Some(decoded_bytes)),
                    Err(e) => Result::Err(e),
                }
            }
            None => Result::Ok(None),
        }
    }
    /// Extracts text from PDF content if the response body is a PDF.
    /// Returns Ok(Some(text)) on successful parsing of PDF,
    /// Ok(None) if no PDF content is present,
    /// or Err(ZyteError) on failure.
    pub fn extract_pdf_text(&self) -> Result<Option<String>, crate::zyte::error::ZyteError> {
        // Attempt to decode the HTTP response body
        let decoded_option = self.decoded_http_response_body()
            .map_err(|e| crate::zyte::error::ZyteError::PdfDecodeError(e.to_string()))?;
        if let Some(bytes) = decoded_option {
            // Check for PDF signature
            if bytes.starts_with(b"%PDF-") {
                // Parse PDF document from memory
                let doc = lopdf::Document::load_mem(&bytes)
                    .map_err(|e| crate::zyte::error::ZyteError::PdfParseError(e.to_string()))?;
                // Extract text from each page
                let mut text = String::new();
                for &page_num in doc.get_pages().keys() {
                    let page_text = doc.extract_text(&[page_num])
                        .map_err(|e| crate::zyte::error::ZyteError::PdfParseError(e.to_string()))?;
                    text.push_str(&page_text);
                    text.push_str("\\n");
                }
                return Ok(Some(text));
            }
        }
        Ok(None)
    }
    /// Cleans HTML boilerplate and extracts main text content from HTML response.
    /// Returns Ok(Some(text)) if HTML content is present,
    /// Ok(None) if no HTML content is available,
    /// or Err(ZyteError) on failure.
    pub fn extract_clean_html(&self) -> Result<Option<String>, crate::zyte::error::ZyteError> {
        // Choose browser_html if available, otherwise use raw http body as UTF-8
        let html_content = if let Some(html) = &self.browser_html {
            html.clone()
        } else if let Some(encoded) = &self.http_response_body {
            // Decode base64
            let bytes = STANDARD.decode(encoded)
                .map_err(|e| crate::zyte::error::ZyteError::HtmlCleanError(e.to_string()))?;
            String::from_utf8_lossy(&bytes).into_owned()
        } else {
            return Ok(None);
        };
        // Parse HTML document
        let document = scraper::Html::parse_document(&html_content);
        // Try selecting <main> or <article> elements
        let selector = scraper::Selector::parse("main, article").unwrap();
        let mut text = String::new();
        for element in document.select(&selector) {
            let snippet = element.text().collect::<Vec<_>>().join(" ");
            if !snippet.trim().is_empty() {
                text.push_str(&snippet);
                text.push_str("\\n");
            }
        }
        // Fallback to all <p> elements if no main/article found
        if text.trim().is_empty() {
            let p_selector = scraper::Selector::parse("p").unwrap();
            for p in document.select(&p_selector) {
                let snippet = p.text().collect::<Vec<_>>().join(" ");
                if !snippet.trim().is_empty() {
                    text.push_str(&snippet);
                    text.push_str("\\n");
                }
            }
        }
        Ok(Some(text))
    }
}

#[cfg(test)]
mod tests {
    // Note: Using ::serde_json for deserialization in test setup.

    #[test]
    fn test_deserialize_and_decode_body_present() {
        // Test deserialization and successful decoding of httpResponseBody.
        let json_input = r#" {
            "url": "http://example.com",
            "statusCode": 200,
            "httpResponseBody": "SGVsbG8gV29ybGQ=", 
            "browserHtml": null,
            "screenshot": null
        } "#;
        let response: Result<super::ZyteResponse, ::serde_json::Error> = ::serde_json::from_str(json_input);
        assert!(response.is_ok());
        let zyte_response = response.unwrap();

        assert_eq!(zyte_response.url, "http://example.com");
        assert_eq!(zyte_response.status_code, 200);
        assert!(zyte_response.http_response_body.is_some());
        assert_eq!(zyte_response.http_response_body.as_deref(), Some("SGVsbG8gV29ybGQ="));

        // Test the decoding method
        let decoded_result = zyte_response.decoded_http_response_body();
        assert!(decoded_result.is_ok());
        let decoded_option = decoded_result.unwrap();
        assert!(decoded_option.is_some());
        let decoded_bytes = decoded_option.unwrap();
        assert_eq!(decoded_bytes, b"Hello World");

        // Convert bytes to String for easier comparison if needed
        let decoded_string = String::from_utf8(decoded_bytes);
        assert!(decoded_string.is_ok());
        assert_eq!(decoded_string.unwrap(), "Hello World");
    }

    #[test]
    fn test_deserialize_and_decode_body_absent() {
        // Test deserialization when httpResponseBody is null.
        let json_input = r#" {
            "url": "http://example.com",
            "statusCode": 404,
            "httpResponseBody": null,
            "browserHtml": null,
            "screenshot": null
        } "#;
        let response: Result<super::ZyteResponse, ::serde_json::Error> = ::serde_json::from_str(json_input);
        assert!(response.is_ok());
        let zyte_response = response.unwrap();

        assert!(zyte_response.http_response_body.is_none());

        // Test the decoding method when body is None
        let decoded_result = zyte_response.decoded_http_response_body();
        assert!(decoded_result.is_ok());
        let decoded_option = decoded_result.unwrap();
        assert!(decoded_option.is_none());
    }

    #[test]
    fn test_decode_invalid_base64() {
        // Test decoding failure with invalid base64.
        let json_input = r#" {
            "url": "http://example.com",
            "statusCode": 200,
            "httpResponseBody": "Invalid Base64!", 
            "browserHtml": null,
            "screenshot": null
        } "#;
        let response: Result<super::ZyteResponse, ::serde_json::Error> = ::serde_json::from_str(json_input);
        assert!(response.is_ok());
        let zyte_response = response.unwrap();

        // Test the decoding method
        let decoded_result = zyte_response.decoded_http_response_body();
        assert!(decoded_result.is_err()); // Expecting a DecodeError
    }
}
