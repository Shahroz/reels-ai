use serde_json::Value;

use tracing::instrument;

#[instrument(skip(content))]
fn find_and_parse_outermost_structure(content: &str) -> String {
    let first_brace = content.find('{');
    let first_bracket = content.find('[');
    let last_brace = content.rfind('}');
    let last_bracket = content.rfind(']');

    let (start, end) = match (first_brace, first_bracket, last_brace, last_bracket) {
        (Some(fb), Some(fbk), Some(lb), Some(lbk)) => {
            // Choose the earliest opening and the latest closing delimiters.
            let start = fb.min(fbk);
            let end = lb.max(lbk) + 1; // Include the closing character.
            (start, end)
        }
        (Some(fb), None, Some(lb), None) => (fb, lb + 1),
        (None, Some(fbk), None, Some(lbk)) => (fbk, lbk + 1),
        _ => return content.to_string(),
    };

    content[start..end].to_string()
}

#[instrument(skip(raw_content))]
pub fn hacky_json_loads(raw_content: &str) -> Option<Value> {
    // First attempt: Direct parsing.
    if let Ok(data) = serde_json::from_str::<Value>(raw_content) {
        return Some(data);
    }

    let content = find_and_parse_outermost_structure(raw_content);

    // Second attempt: Remove newlines.
    let without_newlines = content.replace("\n", " ");
    if let Ok(data) = serde_json::from_str::<Value>(&without_newlines) {
        return Some(data);
    }

    // Third attempt: Replace newlines with escaped newlines.
    let escaped_newlines = content.replace("\n", "\\n");
    if let Ok(data) = serde_json::from_str::<Value>(&escaped_newlines) {
        return Some(data);
    }

    // Fourth attempt: Handle carriage returns as well.
    let modified_text = content.replace("\n", " ").replace("\r", " ");
    if let Ok(data) = serde_json::from_str::<Value>(&modified_text) {
        return Some(data);
    }

    return None;
}

#[cfg(FALSE)]
mod test {
    use crate::integrations::hacky_json_loads::hacky_json_loads;
    use gennodes_datatypes::collection::Collection;
    use gennodes_datatypes::google_ads::GoogleTextAd;

    #[test]
    fn test_parsing() {
        let test_data = include_str!("hacky_json_loads_test_json.json");

        let parsed = hacky_json_loads(test_data).unwrap();
        let collection: Collection<GoogleTextAd> = serde_json::from_value(parsed).unwrap();
        log::debug!("{:#?}", collection);
    }
}