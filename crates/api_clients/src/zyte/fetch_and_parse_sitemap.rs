//! Fetches and parses a sitemap from a URL or local file path, filtering by last modified date.
//!
//! Reads content from the given source (interpreting it as a URL if parsable,
//! otherwise as a file path). Parses the content as a sitemap and extracts
//! URL locations specified within, optionally filtering by a minimum last modification date.
//! Adheres to one-item-per-file guideline. Uses fully qualified paths.
//! Returns a list of URLs or a ZyteError.

//! Revision History
//! - 2025-04-24T07:45:39Z @AI: Correct handling of Option<LastMod> based on compiler error analysis.
//! - 2025-04-24T07:17:50Z @AI: Added min_last_mod filtering logic and updated signature/tests.
//! - 2025-04-24T06:41:14Z @AI: Initial implementation based on instruction.

/// Fetches content from a URL or local file, parses it as a sitemap, and filters URLs by date.
///
/// Checks if `source` is a valid URL. If so, fetches content via HTTP GET.
/// Otherwise, treats `source` as a local file path and reads its content.
/// Parses the content using `sitemap::reader::SiteMapReader`.
/// Extracts URLs, including only those where:
/// - `min_last_mod` is `None`, OR
/// - The sitemap entry has no `lastmod` date (`Option::None`), OR
/// - The entry's `lastmod` date (`Option::Some(LastMod::DateTime)`) is >= `min_last_mod`.
/// - Entries with `lastmod` present but not parsable as `DateTime` (e.g., `LastMod::W3CDateTime`) are included if `min_last_mod` is Some.
///
/// # Arguments
/// * `source` - A string representing either a URL or a local file path.
/// * `min_last_mod` - An optional `DateTime<Utc>` threshold. Entries older than this are excluded.
///
/// # Returns
/// A `Result` containing a `Vec<String>` of filtered URLs found in the sitemap on success,
/// or a `crate::zyte::error::ZyteError` on failure (network, I/O, parsing).
pub async fn fetch_and_parse_sitemap(
    source: &str,
    min_last_mod: Option<::chrono::DateTime<::chrono::Utc>>,
) -> Result<Vec<String>, crate::zyte::error::ZyteError> {
    let sitemap_content: String;

    // Try parsing as URL first
    if let Ok(url) = ::url::Url::parse(source) {
        // Fetch content from URL
        let response = ::reqwest::get(url)
            .await
            .map_err(|e| crate::zyte::error::ZyteError::ReqwestError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(crate::zyte::error::ZyteError::ReqwestError(format!(
                "Failed to fetch sitemap URL: status {}",
                response.status()
            )));
        }
        sitemap_content = response
            .text()
            .await
            .map_err(|e| crate::zyte::error::ZyteError::ReqwestError(e.to_string()))?;
    } else {
        // Treat as local file path
        sitemap_content = ::std::fs::read_to_string(source)
            .map_err(|e| crate::zyte::error::ZyteError::IoError(format!("Failed to read file '{source}': {e}")))?;
    }

    // Parse the sitemap content
    let parser = ::sitemap::reader::SiteMapReader::new(sitemap_content.as_bytes());
    let mut urls = ::std::vec::Vec::new();

    for entity in parser {
        match entity {
            ::sitemap::reader::SiteMapEntity::Url(url_entry) => {
                // Determine if the URL should be included based on the lastmod date filter
                let should_include = match min_last_mod {
                    None => true, // No filter date provided, include all URLs
                    Some(min_date_utc) => {
                        // Filter date provided, check the entry's lastmod
                        match url_entry.lastmod {
                            // No lastmod date for the entry, include it
                            ::sitemap::structs::LastMod::None => true,
                            ::sitemap::structs::LastMod::DateTime(entry_date_fixed) => {
                                // It's a DateTime, compare it
                                let entry_date_utc = entry_date_fixed.with_timezone(&::chrono::Utc);
                                // Include if entry date is on or after the minimum date
                                entry_date_utc >= min_date_utc
                            }
                            // ParseErr or other LastMod variants, include by default
                            _ => true,
                        }
                    }
                };

                if should_include {
                    if let Some(loc) = url_entry.loc.get_url() {
                        urls.push(loc.to_string());
                    }
                }
            }
            ::sitemap::reader::SiteMapEntity::SiteMap(sitemap_entry) => {
                 if let Some(loc) = sitemap_entry.loc.get_url() {
                     // Note: Recursively fetching nested sitemaps is not implemented here.
                     // We just record the sitemap index URL itself if needed, or ignore.
                     // Filtering by date doesn't apply to sitemap index entries themselves directly here.
                     ::std::println!("Found nested sitemap index (not processed recursively): {loc}");
                 }
            }
            ::sitemap::reader::SiteMapEntity::Err(e) => {
                 ::std::eprintln!("Sitemap parsing error: {e}");
                // Return error on first parsing failure
                return Err(crate::zyte::error::ZyteError::SitemapParseError(e.to_string()));
            }
        }
    }

    Ok(urls)
}


#[cfg(test)]
mod tests {
    // Note: Using fully qualified paths as required.
    // Test for URL fetching requires a stable endpoint or mocking.
    // Test for file reading uses temporary files.

    use crate::zyte::fetch_and_parse_sitemap::fetch_and_parse_sitemap;

    // Helper to create a temporary file with content.
    fn create_temp_file(content: &str) -> ::std::path::PathBuf {
        use std::io::Write;
        let filename = format!("test_sitemap_{}.xml", ::rand::random::<u64>());
        let filepath = ::std::env::temp_dir().join(filename);
        let mut file = ::std::fs::File::create(&filepath).expect("Failed to create temp file");
        file.write_all(content.as_bytes()).expect("Failed to write to temp file");
        filepath
    }

    #[tokio::test]
    async fn test_parse_valid_sitemap_no_filter() {
        let sitemap_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
   <url>
      <loc>http://www.example.com/</loc>
      <lastmod>2024-01-01T10:00:00+00:00</lastmod>
   </url>
   <url>
      <loc>http://www.example.com/page1</loc>
      <lastmod>2024-03-15T12:00:00Z</lastmod> <!-- Z indicates UTC -->
   </url>
</urlset>"#;
        let file_path = create_temp_file(sitemap_content);
        let path_str = file_path.to_str().expect("Invalid path string");

        // No date filter applied
        let result = fetch_and_parse_sitemap(path_str, None).await;

        assert!(result.is_ok(), "Result was: {:?}", result);
        let urls = result.unwrap();
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"http://www.example.com/".to_string()));
        assert!(urls.contains(&"http://www.example.com/page1".to_string()));

        ::std::fs::remove_file(file_path).expect("Failed to remove temp file");
    }

     #[tokio::test]
    async fn test_parse_valid_sitemap_with_filter() {
        let sitemap_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
   <url>
      <loc>http://www.example.com/old</loc>
      <lastmod>2024-01-01T10:00:00+00:00</lastmod>
   </url>
   <url>
      <loc>http://www.example.com/new</loc>
      <lastmod>2024-03-15T12:00:00Z</lastmod>
   </url>
   <url>
      <loc>http://www.example.com/no_date</loc>
   </url>
</urlset>"#;
        let file_path = create_temp_file(sitemap_content);
        let path_str = file_path.to_str().expect("Invalid path string");

        // Filter date: Keep entries from 2024-02-01 onwards
        let min_date_str = "2024-02-01T00:00:00Z";
        let min_date = ::chrono::DateTime::parse_from_rfc3339(min_date_str)
            .expect("Failed to parse min_date")
            .with_timezone(&::chrono::Utc);

        let result = fetch_and_parse_sitemap(path_str, Some(min_date)).await;

        assert!(result.is_ok(), "Result was: {:?}", result);
        let urls = result.unwrap();
        // Should include "/new" (after min_date) and "/no_date" (no date = include)
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"http://www.example.com/new".to_string()));
        assert!(urls.contains(&"http://www.example.com/no_date".to_string()));
        assert!(!urls.contains(&"http://www.example.com/old".to_string())); // Should be filtered out

        ::std::fs::remove_file(file_path).expect("Failed to remove temp file");
    }


    #[tokio::test]
    async fn test_parse_invalid_sitemap_from_file() {
        let sitemap_content = r#"<xml>invalid content</xml>"#;
        let file_path = create_temp_file(sitemap_content);
        let path_str = file_path.to_str().expect("Invalid path string");

        let result = fetch_and_parse_sitemap(path_str, None).await;

        assert!(result.is_err());
        match result.err().unwrap() {
            crate::zyte::error::ZyteError::SitemapParseError(_) => { /* Expected */ }
            e => panic!("Expected SitemapParseError, got {:?}", e),
        }

        ::std::fs::remove_file(file_path).expect("Failed to remove temp file");
    }

    #[tokio::test]
    async fn test_file_not_found() {
        let non_existent_path = "/path/that/does/not/exist/sitemap.xml";
        let result = fetch_and_parse_sitemap(non_existent_path, None).await;

        assert!(result.is_err());
        match result.err().unwrap() {
            crate::zyte::error::ZyteError::IoError(_) => { /* Expected */ }
            e => panic!("Expected IoError, got {:?}", e),
        }
    }

    #[tokio::test]
    #[ignore] // Ignored because it makes a real network request. Requires a stable test URL.
    async fn test_fetch_from_url() {
        // Using a known simple sitemap online (replace with a more stable one if needed)
        let url = "https://httpbin.org/xml"; // Provides simple XML content, not a valid sitemap
        let result = fetch_and_parse_sitemap(url, None).await;

        // We expect parsing to fail for httpbin.org/xml as it's not a sitemap,
        // but success means fetching worked initially. A better test would use a real sitemap URL
        // and assert the parsed URLs.
        assert!(result.is_err(), "Expected sitemap parse error for non-sitemap XML, but got Ok or different error: {:?}", result);
         match result.err().unwrap() {
            crate::zyte::error::ZyteError::SitemapParseError(_) => { /* Expected parse error */ }
            // It might also fail earlier during fetch if network is down, so ReqwestError is possible too.
            crate::zyte::error::ZyteError::ReqwestError(_) => { /* Also possible depending on network state */ }
            e => panic!("Expected SitemapParseError or ReqwestError for httpbin XML, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_invalid_url() {
        let invalid_url = "htp:/invalid-url";
        // Since it's not a valid URL, it will be treated as a file path.
        // We expect an IoError because the file won't exist.
        let result = fetch_and_parse_sitemap(invalid_url, None).await;
        assert!(result.is_err());
         match result.err().unwrap() {
            crate::zyte::error::ZyteError::IoError(_) => { /* Expected */ }
            e => panic!("Expected IoError for invalid URL treated as file path, got {:?}", e),
        }
    }
}
