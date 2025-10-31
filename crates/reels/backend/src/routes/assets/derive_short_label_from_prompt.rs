//! Derives a short, human-readable label from an enhancement prompt.
//!
//! Analyzes the prompt text to identify common enhancement patterns
//! (day-to-dusk, virtual staging, renovations, etc.) and returns a concise label.
//! Falls back to truncating the prompt if no pattern matches.
//!
//! Revision History:
//! - 2025-10-17T00:00:00Z @AI: Extracted from enhance_asset.rs

pub fn derive_short_label_from_prompt(prompt: &str) -> std::string::String {
    let p = prompt.to_lowercase();
    let contains = |needle: &str| p.contains(needle);

    if contains("golden-hour") || contains("day to dusk") { return "Day to Dusk".to_string(); }
    if contains("blue hour exterior") || contains("blue hour") { return "Blue Hour Exterior".to_string(); }
    if contains("virtually clean") || contains("tidy") || contains("clutter") { return "Virtual Cleaning".to_string(); }
    if contains("soft styling") { return "Soft Styling".to_string(); }
    if contains("fix lens distortion") || contains("straighten verticals") { return "Straighten Verticals".to_string(); }
    if contains("lift exposure") || contains("dynamic range") { return "Balanced Exposure".to_string(); }
    if contains("remove color casts") || contains("white balance") { return "Neutral White Balance".to_string(); }
    if contains("restore exterior window view") { return "Window View".to_string(); }
    if contains("replace overcast sky") { return "Sky: Subtle Blue".to_string(); }
    if contains("green up grass") { return "Lawn Refresh".to_string(); }
    if contains("clean driveway") || contains("oil spots") { return "Driveway Clean".to_string(); }
    if contains("remove personal items") { return "Remove Personal Items".to_string(); }
    if contains("repair wall scuffs") { return "Wall Touch-Up".to_string(); }
    if contains("turn on interior lights") { return "Lights On (Warm)".to_string(); }
    if contains("reduce distracting reflections") { return "Cut Reflections".to_string(); }
    if contains("visible tv") || contains("monitor screens") || contains("screens to black") { return "Screens to Black".to_string(); }
    if contains("remove rain") || contains("puddles") || contains("rain ") { return "Rain → Dry".to_string(); }
    if contains("reduce noise") && contains("detail") { return "Clean + Detail".to_string(); }

    // Stage / Renovate / Paint patterns
    if let Some(style) = p.strip_prefix("virtually stage in ").and_then(|s| s.split(" style").next()) {
        let s_cap = style.trim().split_whitespace().map(|w| {
            let mut c = w.chars();
            match c.next() { Some(f) => f.to_uppercase().collect::<std::string::String>() + c.as_str(), None => std::string::String::new() }
        }).collect::<std::vec::Vec<_>>().join(" ");
        return format!("Stage: {}", s_cap);
    }
    if let Some(style) = p.strip_prefix("virtually renovate to ").and_then(|s| s.split(" style").next()) {
        let s_cap = style.trim().split_whitespace().map(|w| {
            let mut c = w.chars();
            match c.next() { Some(f) => f.to_uppercase().collect::<std::string::String>() + c.as_str(), None => std::string::String::new() }
        }).collect::<std::vec::Vec<_>>().join(" ");
        return format!("Renovate: {}", s_cap);
    }
    if let Some(color) = p.strip_prefix("change wall paint to ").and_then(|s| s.split(';').next()) {
        let s_cap = color.trim().split_whitespace().map(|w| {
            let mut c = w.chars();
            match c.next() { Some(f) => f.to_uppercase().collect::<std::string::String>() + c.as_str(), None => std::string::String::new() }
        }).collect::<std::vec::Vec<_>>().join(" ");
        return format!("Paint: {}", s_cap);
    }

    // Fallback - truncate prompt sensibly
    let s = prompt.trim();
    if s.is_empty() { return "Derived".to_string(); }
    let max_len = 28usize;
    let mut out = s.chars().take(max_len).collect::<std::string::String>();
    if s.len() > out.len() { out.push_str("…"); }
    if let Some(first) = out.get(0..1) { out.replace_range(0..1, &first.to_uppercase().to_string()); }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_day_to_dusk() {
        assert_eq!(derive_short_label_from_prompt("golden-hour transformation"), "Day to Dusk");
        assert_eq!(derive_short_label_from_prompt("day to dusk conversion"), "Day to Dusk");
    }

    #[test]
    fn test_blue_hour() {
        assert_eq!(derive_short_label_from_prompt("blue hour exterior lighting"), "Blue Hour Exterior");
        assert_eq!(derive_short_label_from_prompt("BLUE HOUR effect"), "Blue Hour Exterior");
    }

    #[test]
    fn test_virtual_cleaning() {
        assert_eq!(derive_short_label_from_prompt("virtually clean the room"), "Virtual Cleaning");
        assert_eq!(derive_short_label_from_prompt("tidy up the space"), "Virtual Cleaning");
        assert_eq!(derive_short_label_from_prompt("remove clutter"), "Virtual Cleaning");
    }

    #[test]
    fn test_staging_patterns() {
        assert_eq!(derive_short_label_from_prompt("virtually stage in modern style"), "Stage: Modern");
        assert_eq!(derive_short_label_from_prompt("virtually stage in scandinavian style"), "Stage: Scandinavian");
    }

    #[test]
    fn test_renovation_patterns() {
        assert_eq!(derive_short_label_from_prompt("virtually renovate to contemporary style"), "Renovate: Contemporary");
        assert_eq!(derive_short_label_from_prompt("virtually renovate to industrial style"), "Renovate: Industrial");
    }

    #[test]
    fn test_paint_patterns() {
        assert_eq!(derive_short_label_from_prompt("change wall paint to warm beige;"), "Paint: Warm Beige");
        assert_eq!(derive_short_label_from_prompt("change wall paint to cool gray; other params"), "Paint: Cool Gray");
    }

    #[test]
    fn test_specific_enhancements() {
        assert_eq!(derive_short_label_from_prompt("restore exterior window view"), "Window View");
        assert_eq!(derive_short_label_from_prompt("replace overcast sky"), "Sky: Subtle Blue");
        assert_eq!(derive_short_label_from_prompt("green up grass and lawn"), "Lawn Refresh");
        assert_eq!(derive_short_label_from_prompt("remove personal items from photo"), "Remove Personal Items");
        assert_eq!(derive_short_label_from_prompt("turn on interior lights"), "Lights On (Warm)");
    }

    #[test]
    fn test_empty_prompt() {
        assert_eq!(derive_short_label_from_prompt(""), "Derived");
        assert_eq!(derive_short_label_from_prompt("   "), "Derived");
    }

    #[test]
    fn test_truncation_fallback() {
        let long_prompt = "This is a very long custom prompt that does not match any specific pattern and should be truncated";
        let result = derive_short_label_from_prompt(long_prompt);
        // The function takes 28 chars + adds ellipsis (which is 3 bytes but 1 character)
        // The max length depends on character boundaries
        assert!(result.len() <= 32); // Allow for UTF-8 ellipsis byte length
        assert!(result.ends_with("…"));
        // First char should be uppercase
        assert!(result.chars().next().unwrap().is_uppercase());
        // Verify it's actually truncated
        assert!(result.chars().count() <= 29); // 28 chars + ellipsis
    }

    #[test]
    fn test_short_custom_prompt() {
        assert_eq!(derive_short_label_from_prompt("custom edit"), "Custom edit");
    }

    #[test]
    fn test_case_insensitivity() {
        assert_eq!(derive_short_label_from_prompt("DAY TO DUSK"), "Day to Dusk");
        assert_eq!(derive_short_label_from_prompt("Blue HOUR"), "Blue Hour Exterior");
        assert_eq!(derive_short_label_from_prompt("VIRTUALLY CLEAN"), "Virtual Cleaning");
    }
}

