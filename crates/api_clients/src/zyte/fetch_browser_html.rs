//! Fetches the rendered browser HTML content of a given URL using the Zyte API.
//!
//! This function utilizes the `ZyteClient` to request the `browserHtml` field
//! for the specified URL. It handles client creation, request building,
//! and extracting the relevant part of the Zyte API response.
//! Returns an Option<String> containing the HTML, or a ZyteError.
//! Conforms to the project's Rust guidelines (fully qualified paths, preamble, etc.).
//! Revision History
//! - 2025-05-02T12:16:09Z @AI: Extract internal links from raw HTML and append to content.
//! - 2025-05-02T04:59:40Z @AI: Integrated dom_smoothie for text extraction.
//! - 2025-05-02T03:54:52Z @AI: Initial implementation based on refactoring fetch_text.

// Use fully qualified paths as per guidelines
// use crate::zyte::simplified_article::SimplifiedArticle; // Use FQN below
// use dom_smoothie::{Article, Config, Readability}; // Use FQN below
// use scraper::{Html, Selector}; // Use FQN below
// use url::Url; // Use FQN below
// use std::collections::HashSet; // Use FQN below

/// Fetches rendered browser HTML from a URL via Zyte API.
///
/// # Arguments
/// * `url` - The URL to fetch HTML for.
///
/// # Returns
/// * `Ok(SimplifiedArticle)` containing the extracted article on success.
/// * `Err(crate::zyte::error::ZyteError)` on failure (client error, API error, etc.).
pub async fn fetch_browser_html_and_extract_text(url: &str) -> std::result::Result<crate::zyte::simplified_article::SimplifiedArticle, crate::zyte::error::ZyteError> {
    let _ = dotenv::dotenv();
    // Create a ZyteClient instance
    let client = crate::zyte::client::ZyteClient::new()?; // Propagate ZyteError

    // Create a ZyteRequest payload
    let request = crate::zyte::request::ZyteRequest {
        url: url.to_string(),
        http_response_body: std::option::Option::None, // Not needed
        browser_html: std::option::Option::Some(true), // Request browser HTML
        screenshot: std::option::Option::None,
        screenshot_options: std::option::Option::None,
        actions: std::option::Option::None,
    };

    // Perform the extraction request
    let response = client.extract(&request).await?; // Propagate ZyteError

    // Process browserHtml with dom_smoothie if available
    if let Some(raw_html) = response.browser_html {
        // --- Step 1: Extract main content using dom_smoothie ---
        // Use default dom_smoothie config for now, can be customized later
        // Parse the base URL once for use with both dom_smoothie and link extraction
        let base_url = ::url::Url::parse(url)
            .map_err(|e| crate::zyte::error::ZyteError::HtmlCleanError(e.to_string()))?;

        // dom_smoothie expects the base URL as a string slice
        let base_url_string = base_url.to_string();
        let mut readability = ::dom_smoothie::Readability::new(raw_html.to_string(), Some(&base_url_string), None)
            .map_err(|e| crate::zyte::error::ZyteError::HtmlCleanError(e.to_string()))?;

        let article: ::dom_smoothie::Article = readability.parse()
            .map_err(|e| crate::zyte::error::ZyteError::HtmlCleanError(e.to_string()))?;

        // Use the 'content' field which might contain HTML (as opposed to 'text_content')
        let main_content: String = article.content.to_string(); // Note: dom_smoothie's 'content' might contain HTML tags.

        // --- Step 2: Extract and clean internal links from raw HTML ---
        let base_host = base_url.host_str().unwrap_or("").to_lowercase();

        let document = ::scraper::Html::parse_document(&raw_html);
        let link_selector = ::scraper::Selector::parse("a[href]").unwrap(); // Safe unwrap for static selector
        let mut internal_links = ::std::collections::HashSet::new();

        for element in document.select(&link_selector) {
            if let Some(href) = element.value().attr("href") {
                // Attempt to join the href with the base URL
                if let Ok(mut link_url) = base_url.join(href) {
                    // Clean the URL: remove fragment and query parameters
                    link_url.set_fragment(None);
                    link_url.set_query(None);

                    // Check if it's internal (same host)
                    // Note: url::Url::join resolves relative paths correctly relative to the base URL
                    let link_host = link_url.host_str().unwrap_or("").to_lowercase();

                    // Consider it internal if host matches OR if the original href was clearly relative
                    // (starts with '/', '.', or contains no scheme/host)
                    let is_relative_href = href.starts_with('/') || href.starts_with('.') || (!href.contains(':') && !href.starts_with("//"));

                    if link_host == base_host || (is_relative_href && link_host.is_empty()) { // Handle relative paths that might not get a host if base is file:// etc.
                         // Avoid adding the base URL itself (after cleaning) if it appears as a link
                         if link_url != base_url {
                            internal_links.insert(link_url.to_string());
                         }
                    }
                }
            }
        }

        let links_string = if !internal_links.is_empty() {
            let mut links_list: Vec<String> = internal_links.into_iter().collect();
            links_list.sort(); // Sort for consistent output
            format!("\n\n--- Internal Links ---\n{}", links_list.join("\n"))
        } else {
            String::new() // No links found, append nothing
        };

        // Combine content and links
        // Note: We are now appending links to `article.content` which might contain HTML.
        // If plain text is strictly required, `article.text_content` should be used
        // and potentially cleaned like before, but the request was to append links.
        let final_content = format!("{main_content}{links_string}");

        let simplified = crate::zyte::simplified_article::SimplifiedArticle {
            title: article.title,
            content: final_content, // Use combined content + links
            byline: article.byline,
            excerpt: article.excerpt,
            site_name: article.site_name,
            lang: article.lang,
            url: article.url, // Use the one from dom_smoothie if available
        };
        std::result::Result::Ok(simplified)
    } else {
        // No browser HTML returned from Zyte
        std::result::Result::Err(crate::zyte::error::ZyteError::NoBrowserHtml("No browser HTML returned from Zyte".to_string()))
    }
}

#[cfg(test)]
mod tests {
    // Note: These tests require a valid ZYTE_API_KEY environment variable to succeed
    // when running against the actual API. Mocking would be needed for isolated unit tests.

    #[tokio::test]
    #[ignore] // Ignored by default as it requires a live API key and network access
    async fn test_fetch_real_url_2() {
        // Test fetching a real URL. Requires ZYTE_API_KEY.
        ::dotenvy::dotenv().ok(); // Load .env if present
        if ::std::env::var("ZYTE_API_KEY").is_err() {
            println!("Skipping test_fetch_real_url as ZYTE_API_KEY is not set.");
            return;
        }

        let url = "https://bounti.ai/blog";
        let result = super::fetch_browser_html_and_extract_text(url).await;
        println!("{:?}", result);

    }

    // use crate::zyte::article::SimplifiedArticle; // Use FQN below if needed, or rely on super::
    #[tokio::test]
    #[ignore] // Ignored by default as it requires a live API key and network access
    async fn test_fetch_real_url() {
        // Test fetching a real URL. Requires ZYTE_API_KEY.
        ::dotenvy::dotenv().ok(); // Load .env if present
        if ::std::env::var("ZYTE_API_KEY").is_err() {
            println!("Skipping test_fetch_real_url as ZYTE_API_KEY is not set.");
            return;
        }

        let url = "http://books.toscrape.com/";
        let result = super::fetch_browser_html_and_extract_text(url).await;
        println!("{:?}", result);

        assert!(result.is_ok(), "fetch_browser_html failed: {:?}", result);
        let article = result.unwrap();

        // Check some fields of the extracted article
        assert!(!article.title.is_empty(), "Article title should not be empty");
        assert!(article.content.len() > 50, "Article content should have significant length"); // Basic check
        // Check if internal links section likely exists (crude check)
        assert!(article.content.contains("--- Internal Links ---"), "Expected internal links section marker");
        assert!(article.content.contains("http://books.toscrape.com/catalogue/"), "Expected internal links for books.toscrape.com");
    }

    #[tokio::test]
    #[ignore] // Ignored by default as it requires a live API key and network access
    async fn test_fetch_real_url_3() {
        // Test fetching a real URL. Requires ZYTE_API_KEY.
        ::dotenvy::dotenv().ok(); // Load .env if present
        if ::std::env::var("ZYTE_API_KEY").is_err() {
            println!("Skipping test_fetch_real_url_2 as ZYTE_API_KEY is not set.");
            return;
        }

        let url = "https://bounti.ai/blog";
        let result = super::fetch_browser_html_and_extract_text(url).await;
        println!("{:?}", result);

        assert!(result.is_ok(), "fetch_browser_html failed: {:?}", result);
        let article = result.unwrap();

        // Check some fields of the extracted article
        assert!(!article.title.is_empty(), "Article title should not be empty");
        assert!(article.content.len() > 50, "Article content should have significant length"); // Basic check
        // Bounti.ai might have fewer obvious internal links, check presence of marker or substantial content
        // It might also have external links only, so check for marker OR if content is much larger than title
        let has_links_marker = article.content.contains("--- Internal Links ---");
        let has_substantial_content = article.content.len() > article.title.len() + 50;
        assert!(has_links_marker || has_substantial_content, "Content should contain links section or be substantially longer than title. Content: '{}'", article.content);
    }

    #[tokio::test]
    async fn test_invalid_api_key_scenario() {
         // This test assumes the API key is *invalid* or *missing* to test error handling.
         // Temporarily unset the key for this test's scope if possible, or ensure it's not set globally.
         let original_key = std::env::var("ZYTE_API_KEY").ok();
         std::env::remove_var("ZYTE_API_KEY"); // Temporarily unset

         let url = "http://example.com"; // URL doesn't matter much here
         let result = super::fetch_browser_html_and_extract_text(url).await;

         assert!(result.is_err(), "Expected an error due to missing/invalid API key, but got Ok");
         // Check for specific error type if needed (e.g., EnvVarError or AuthenticationError)
         match result.err().unwrap() {
             crate::zyte::error::ZyteError::EnvVarError(_) => { /* Expected */ } ,
             // Depending on ZyteClient::new implementation, it might bubble up reqwest errors too
             crate::zyte::error::ZyteError::ReqwestError(_) => { /* Possible if client creation involves a check */ },
             crate::zyte::error::ZyteError::AuthenticationError(_) => { /* Also possible if client creation succeeded but auth failed */ },
             e => panic!("Expected EnvVarError, ReqwestError or AuthenticationError, but got {:?}", e),
         }

         // Restore original key if it existed
         if let Some(key) = original_key {
             std::env::set_var("ZYTE_API_KEY", key);
         }
    }

    // Add more tests: e.g., for URLs that might cause Zyte errors,
    // or scenarios where Zyte might not return browserHtml.
}
