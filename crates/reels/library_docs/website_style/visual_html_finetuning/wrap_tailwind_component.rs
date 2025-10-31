const TAILWIND_LINK: &str = r#"<script src="https://cdn.tailwindcss.com"></script>"#;
pub fn wrap_tailwind_component(html: &str) -> String {
    // Use a consistent Tailwind import URL.
    let mut result = html.to_string();

    // 1. Ensure the content is wrapped in <html> ... </html>
    if !result.trim_start().starts_with("<html>") {
        result = format!("<html>{}</html>", result);
    }

    // 2. Ensure there is a <head> section.
    //    If missing, insert an empty <head> immediately after <html>.
    if !result.contains("<head>") {
        result = result.replacen("<html>", "<html><head></head>", 1);
    }

    // 3. Ensure the Tailwind import is present in the <head>.
    //    (We check for known Tailwind CDNs to decide if it’s already included.)
    if let Some(head_start) = result.find("<head>") {
        if let Some(head_end) = result.find("</head>") {
            let head_section = &result[head_start..head_end];
            if !(head_section.contains("cdn.tailwindcss.com") || head_section.contains("cdn.jsdelivr.net")) {
                // Insert the Tailwind link immediately after the opening <head> tag.
                let insert_index = head_start + "<head>".len();
                result.insert_str(insert_index, TAILWIND_LINK);
            }
        }
    }

    // 4. Ensure there is a <body> section.
    //    If <body> is missing, wrap all content after </head> in <body> ... </body>.
    if !result.contains("<body>") {
        if let Some(head_end_idx) = result.find("</head>") {
            if let Some(html_end_idx) = result.rfind("</html>") {
                let before_body = &result[..head_end_idx + "</head>".len()];
                let body_content = &result[head_end_idx + "</head>".len()..html_end_idx];
                let after_html = &result[html_end_idx..];
                result = format!("{}<body>{}</body>{}", before_body, body_content, after_html);
            } else {
                // Fallback in case </html> is not found.
                result = result.replace("<html>", "<html><body>").replace("</html>", "</body></html>");
            }
        } else {
            // Fallback if no </head> is found.
            result = result.replace("<html>", "<html><body>").replace("</html>", "</body></html>");
        }
    }

    result
}

/// Reverse the wrapping performed by `wrap_tailwind_component`.
///
/// It will:
/// - If a `<body>`…`</body>` is found, return its inner content.
/// - Otherwise, if the content is wrapped in `<html>` tags, remove them and any `<head>` section.
/// - Otherwise, return the input unchanged.
pub fn unwrap_tailwind_component(html: &str) -> String {
    // If the content contains a <body> section, extract its inner content.
    if let (Some(body_start), Some(body_end)) = (html.find("<body>"), html.find("</body>")) {
        let start = body_start + "<body>".len();
        return html[start..body_end].to_string();
    }

    // Otherwise, if wrapped in <html> tags, remove the outer tags.
    if html.trim_start().starts_with("<html>") && html.trim_end().ends_with("</html>") {
        let inner = &html["<html>".len()..html.len() - "</html>".len()];
        // If a <head> section exists, remove it.
        if let (Some(head_start), Some(head_end)) = (inner.find("<head>"), inner.find("</head>")) {
            let after_head = &inner[head_end + "</head>".len()..];
            return after_head.to_string();
        }
        return inner.to_string();
    }

    // If no known wrapper is detected, return the input unchanged.
    html.to_string()
}

#[cfg(FALSE)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_input() {
        let input = "<div>{{TEMPLATE}}</div>";
        let expected = format!(
            "<html><head>{}</head><body><div>{{{{TEMPLATE}}}}</div></body></html>",
            TAILWIND_LINK
        );
        assert_eq!(wrap_tailwind_component(input), expected);
    }

    #[test]
    fn test_missing_head() {
        let input = "<html><body>{{TEMPLATE}}</body></html>";
        let expected = format!(
            "<html><head>{}</head><body>{{{{TEMPLATE}}}}</body></html>",
            TAILWIND_LINK
        );
        assert_eq!(wrap_tailwind_component(input), expected);
    }

    #[test]
    fn test_missing_body() {
        let input = "<html><head></head>{{TEMPLATE}}</html>";
        let expected = format!(
            "<html><head>{}</head><body>{{{{TEMPLATE}}}}</body></html>",
            TAILWIND_LINK
        );
        assert_eq!(wrap_tailwind_component(input), expected);
    }

    #[test]
    fn test_complete_structure_without_tailwind() {
        let input = "<html><head></head><body>{{TEMPLATE}}</body></html>";
        let expected = format!(
            "<html><head>{}</head><body>{{{{TEMPLATE}}}}</body></html>",
            TAILWIND_LINK
        );
        assert_eq!(wrap_tailwind_component(input), expected);
    }

    #[test]
    fn test_complete_structure_with_tailwind() {
        let input = "<html><head><link href=\"https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css\" rel=\"stylesheet\"></head><body>{{TEMPLATE}}</body></html>";
        // When Tailwind is already present, the function should leave the input unchanged.
        assert_eq!(wrap_tailwind_component(input), input);
    }

    #[test]
    fn test_tailwind_in_comment() {
        let input = "<html><head><!-- tailwind --></head><body>{{TEMPLATE}}</body></html>";
        let expected = format!(
            "<html><head>{}<!-- tailwind --></head><body>{{{{TEMPLATE}}}}</body></html>",
            TAILWIND_LINK
        );
        assert_eq!(wrap_tailwind_component(input), expected);
    }

    #[test]
    fn test_malformed_html() {
        let input = "<div><span>{{TEMPLATE}}</div></span>";
        let expected = format!(
            "<html><head>{}</head><body><div><span>{{{{TEMPLATE}}}}</div></span></body></html>",
            TAILWIND_LINK
        );
        assert_eq!(wrap_tailwind_component(input), expected);
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        let expected = format!(
            "<html><head>{}</head><body></body></html>",
            TAILWIND_LINK
        );
        assert_eq!(wrap_tailwind_component(input), expected);
    }

    #[test]
    fn test_multiple_head_tags() {
        let input = "<html><head></head><head></head><body>{{TEMPLATE}}</body></html>";
        let expected = format!(
            "<html><head>{}</head><head></head><body>{{{{TEMPLATE}}}}</body></html>",
            TAILWIND_LINK
        );
        assert_eq!(wrap_tailwind_component(input), expected);
    }

    #[test]
    fn test_minimal_input_unwarp() {
        let input = "<div>{{TEMPLATE}}</div>";
        let wrapped = wrap_tailwind_component(input);
        assert_eq!(unwrap_tailwind_component(&wrapped), input);
    }

    #[test]
    fn test_complete_structure() {
        let input = "<html><head><link href=\"https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css\" rel=\"stylesheet\"></head><body>{{TEMPLATE}}</body></html>";
        // The wrapping function would not modify this input because Tailwind is already present.
        // Unwrapping should extract only the inner template.
        assert_eq!(unwrap_tailwind_component(input), "{{TEMPLATE}}");
    }

    #[test]
    fn test_unwrap_malformed_html() {
        let input = "<div><span>{{TEMPLATE}}</div></span>";
        let wrapped = wrap_tailwind_component(input);
        assert_eq!(unwrap_tailwind_component(&wrapped), input);
    }

    #[test]
    fn test_unwrap_empty_input() {
        let input = "";
        let wrapped = wrap_tailwind_component(input);
        assert_eq!(unwrap_tailwind_component(&wrapped), "");
    }
}
