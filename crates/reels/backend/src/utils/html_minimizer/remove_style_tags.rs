//! Provides a function to remove style tags from an HTML string.
//!
//! This function parses an HTML string and removes all occurrences of `<style>` tags,
//! including their content. It uses the `kuchikiki` crate to robustly parse
//! and manipulate the HTML tree.

pub fn remove_style_tags(html: &str) -> std::string::String {
    if html.trim().is_empty() {
        return std::string::String::from(html);
    }

    let document = kuchikiki::traits::TendrilSink::one(kuchikiki::parse_html(), html);

    let mut nodes_to_remove = std::vec::Vec::new();
    if let Ok(selector) = document.select("style") {
        for node_match in selector {
            nodes_to_remove.push(node_match.as_node().clone());
        }
    }

    if let Ok(selector) = document.select("script") {
        for node_match in selector {
            nodes_to_remove.push(node_match.as_node().clone());
        }
    }

    for node in nodes_to_remove {
        node.detach();
    }

    document.to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_remove_single_style_tag() {
        let html = "<html><head><style>body { color: red; }</style></head><body><p>Hello</p></body></html>";
        let expected = "<html><head></head><body><p>Hello</p></body></html>";
        let result = super::remove_style_tags(html);
        std::assert_eq!(result, expected);
    }

    #[test]
    fn test_remove_multiple_style_tags() {
        let html = "<style>p { font-size: 12px; }</style><div>text</div><style>div { border: 1px solid black; }</style>";
        let expected = "<html><head></head><body><div>text</div></body></html>";
        let result = super::remove_style_tags(html);
        std::assert_eq!(result, expected);
    }

    #[test]
    fn test_no_style_tags() {
        let html = "<div><p>Some content</p></div>";
        let expected = "<html><head></head><body><div><p>Some content</p></div></body></html>";
        let result = super::remove_style_tags(html);
        std::assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_html_string() {
        let html = "";
        let result = super::remove_style_tags(html);
        std::assert_eq!(result, "");
    }

    #[test]
    fn test_whitespace_html_string() {
        let html = "   ";
        let result = super::remove_style_tags(html);
        std::assert_eq!(result, "   ");
    }

    #[test]
    fn test_style_tag_with_attributes() {
        let html = r#"<style type="text/css" media="screen">h1 { font-weight: bold; }</style><h1>Title</h1>"#;
        let expected = r#"<html><head></head><body><h1>Title</h1></body></html>"#;
        let result = super::remove_style_tags(html);
        std::assert_eq!(result, expected);
    }
}
