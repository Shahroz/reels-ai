//! Filters a slice of URL strings based on an optional query substring.
//!
//! This function takes a slice of URLs and an optional filter string.
//! It filters the input slice, keeping only URLs that contain the query substring.
//! If the filter query is None, a clone of the original slice is returned.
//! Adheres to Rust guidelines requiring FQN where necessary and specific documentation style.

//! Revision History
//! - 2025-04-24T07:00:34Z @AI: Updated signature, logic, and tests per user request.

/// Filters URLs based on an optional query string.
///
/// # Arguments
///
/// * `urls` - A slice of strings representing the URLs to filter.
/// * `filter_query` - An optional string slice. If Some, URLs containing this substring are kept.
///
/// # Returns
///
/// A `Vec<String>` containing the filtered URLs. If `filter_query` is None,
/// returns a vector containing clones of all URLs in the input slice.
pub fn filter_sitemap_urls(urls: &[String], filter_query: Option<&str>) -> Vec<String> {
    match filter_query {
        // If a query is provided, filter the URLs.
        Some(query) => urls
            .iter()
            .filter(|url| url.contains(query))
            .cloned() // Clone the filtered URLs to create a new Vec<String>
            .collect(),
        // If no query is provided, clone all URLs from the slice.
        None => urls.to_vec(), // .to_vec() clones the elements
    }
}

#[cfg(test)]
mod tests {
    // Access the function under test via `super::`. Use prelude types where possible.

    #[test]
    fn test_filter_with_query() {
        // Test filtering with a specific query substring.
        let urls = vec![
            String::from("https://example.com/page1"),
            String::from("https://example.com/section/page2"),
            String::from("https://anotherexample.com/page1"),
        ];
        let query = Some("section");
        let expected = vec![String::from(
            "https://example.com/section/page2",
        )];
        let result = super::filter_sitemap_urls(&urls, query);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_filter_with_no_query() {
        // Test behavior when no filter query is provided (None).
        let urls = vec![
            String::from("https://example.com/page1"),
            String::from("https://example.com/section/page2"),
        ];
        let query = None;
        let expected = urls.clone(); // Expect the original list back
        let result = super::filter_sitemap_urls(&urls, query);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_filter_empty_url_list() {
        // Test filtering with an empty input list of URLs.
        let urls: Vec<String> = vec![];
        let query_some = Some("test");
        let query_none = None;
        let expected: Vec<String> = vec![];

        // Test with Some query on empty list
        let result_some = super::filter_sitemap_urls(&urls, query_some);
        assert_eq!(result_some, expected, "Filtering empty list with Some query failed");

        // Test with None query on empty list
        let result_none = super::filter_sitemap_urls(&urls, query_none);
        assert_eq!(result_none, expected, "Filtering empty list with None query failed");
    }

    #[test]
    fn test_filter_query_matches_none() {
        // Test filtering when the query matches no URLs in the list.
        let urls = vec![
            String::from("https://example.com/page1"),
            String::from("https://example.com/page2"),
        ];
        let query = Some("nomatch");
        let expected: Vec<String> = vec![];
        let result = super::filter_sitemap_urls(&urls, query);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_filter_query_matches_all() {
        // Test filtering when the query matches all URLs in the list.
        let urls = vec![
            String::from("https://test.com/page1"),
            String::from("https://test.com/page2"),
        ];
        let query = Some("test.com");
        let expected = urls.clone();
        let result = super::filter_sitemap_urls(&urls, query);
        assert_eq!(result, expected);
    }

     #[test]
    fn test_case_sensitivity() {
        // Test that filtering is case-sensitive.
        let urls = vec![
            String::from("https://example.com/PAGE"),
            String::from("https://example.com/page"),
        ];
        let query = Some("page"); // Lowercase query
        let expected = vec![
            String::from("https://example.com/page"), // Should only match lowercase
        ];
        let result = super::filter_sitemap_urls(&urls, query);
        assert_eq!(result, expected);
    }
}
