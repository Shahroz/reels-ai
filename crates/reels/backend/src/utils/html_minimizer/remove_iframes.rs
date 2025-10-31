//! Removes all iframe elements from an HTML string.
//!
//! This function parses the given HTML content and removes any `<iframe>` tags,
//! returning the cleaned HTML. It is useful for sanitizing user-provided
//! HTML or minimizing content for specific display purposes. It leverages
//! the `kuchikiki` crate for robust HTML parsing and manipulation.

use kuchikiki::traits::TendrilSink;

pub fn remove_iframes(html: &str) -> std::string::String {
    if html.trim().is_empty() {
        return std::string::String::new();
    }

    let doc = kuchikiki::parse_html().one(html);

    // Collect NodeRefs of iframes to detach.
    // This avoids issues with modifying the DOM tree while iterating over matches from doc.select(),
    // which could lead to skipping some elements if the collection is live.
    let iframes_to_detach: std::vec::Vec<kuchikiki::NodeRef> = doc
        .select("iframe")
        .map_or(std::vec::Vec::new(), |matches| {
            matches.map(|data_ref| data_ref.as_node().clone()).collect()
        });

    for iframe_node in iframes_to_detach {
        iframe_node.detach();
    }

    // Serialize the modified document back to a string.
    doc.to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_remove_multiple_iframes() {
        let html = "<iframe></iframe><p>text</p><iframe></iframe>";
        let expected = "<html><head></head><body><p>text</p></body></html>";
        let result = super::remove_iframes(html);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_no_iframes() {
        let html = "<div><p>Hello World</p></div>";
        let expected = "<html><head></head><body><div><p>Hello World</p></div></body></html>";
        let result = super::remove_iframes(html);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_iframe_with_content() {
        let html = "<p>start</p><iframe src='...'><div>Some content</div></iframe><p>end</p>";
        let expected = "<html><head></head><body><p>start</p><p>end</p></body></html>";
        let result = super::remove_iframes(html);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_string() {
        let html = "";
        let expected = "";
        let result = super::remove_iframes(html);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_malformed_html_still_parses() {
        // kuchikiki is robust and will parse this.
        let html = "<p>Hello <iframe src='...'> <b>World</b>";
        // If "<b>World</b>" is parsed as fallback content for the iframe (which is likely for
        // unclosed/malformed HTML like this), it should be removed along with the iframe.
        let expected = "<html><head></head><body><p>Hello </p></body></html>";
        let result = super::remove_iframes(html);
        assert_eq!(result, expected);
    }
}
