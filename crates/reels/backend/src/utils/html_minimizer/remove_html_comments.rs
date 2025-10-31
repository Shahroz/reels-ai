//! Removes all HTML comments from a string.
//!
//! Removes all HTML comments from a string.
//!
//! This function parses an HTML string using the `kuchikiki` crate, traverses the
//! document's nodes, and removes any comment nodes. It is designed to handle
//! HTML fragments as well as full documents.

pub fn remove_html_comments(html: &str) -> std::string::String {
    let document = kuchikiki::traits::TendrilSink::one(kuchikiki::parse_html(), html);

    let mut nodes_to_remove = std::vec::Vec::new();
    document.traverse().for_each(|edge| {
        if let kuchikiki::iter::NodeEdge::Start(node_ref) = edge {
            if node_ref.as_comment().is_some() {
                nodes_to_remove.push(node_ref);
            }
        }
    });

    for node in nodes_to_remove {
        node.detach();
    }

    document.to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_remove_basic_comment() {
        let html = "<div><!-- comment -->Hello</div>";
        let expected = "<html><head></head><body><div>Hello</div></body></html>";
        let result = super::remove_html_comments(html);
        std::assert_eq!(result, expected);
    }

    #[test]
    fn test_remove_multiple_comments() {
        let html = "<!-- c1 --><div><!-- c2 -->Hello<!-- c3 --></div><!-- c4 -->";
        let expected = "<html><head></head><body><div>Hello</div></body></html>";
        let result = super::remove_html_comments(html);
        std::assert_eq!(result, expected);
    }

    #[test]
    fn test_no_comments() {
        let html = "<div><p>Hello, world!</p></div>";
        let result = super::remove_html_comments(html);
        let expected = "<html><head></head><body><div><p>Hello, world!</p></div></body></html>";
        std::assert_eq!(result, expected);
    }

    #[test]
    fn test_only_comment() {
        let html = "<!-- just a comment -->";
        let expected = "<html><head></head><body></body></html>";
        let result = super::remove_html_comments(html);
        std::assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_string() {
        let html = "";
        let expected = "<html><head></head><body></body></html>";
        let result = super::remove_html_comments(html);
        std::assert_eq!(result, expected);
    }

    #[test]
    fn test_html_inside_comment() {
        // HTML comments don't nest. The first `-->` closes the comment.
        let html = "<div><!-- <span>ignore me</span> -->Hello</div>";
        let expected = "<html><head></head><body><div>Hello</div></body></html>";
        let result = super::remove_html_comments(html);
        std::assert_eq!(result, expected);
    }

    #[test]
    fn test_adjacent_text_and_element() {
        let html = "text before <div>div</div> text after";
        let result = super::remove_html_comments(html);
        let expected = "<html><head></head><body>text before <div>div</div> text after</body></html>";
        std::assert_eq!(result, expected);
    }
}
