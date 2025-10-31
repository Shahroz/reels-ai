//! Extracts CSS color values from HTML content.
//!
//! This module provides a function `extract_colors_from_html` that uses regular expressions
//! to identify various CSS color formats (hexadecimal, RGB/RGBA, HSL/HSLA, named colors)
//! within HTML content. All found colors are converted to a standard 6-digit lowercase
//! hexadecimal format (e.g., "#rrggbb") and returned as a unique set.

// Helper function to normalize hex color strings to #RRGGBB format.
// Input hex_str is assumed to be lowercase and start with '#'.
fn normalize_hex_input(hex_str: &str) -> std::option::Option<std::string::String> {
    if !hex_str.starts_with('#') {
        return std::option::Option::None;
    }
    let hex_part = &hex_str[1..]; // Remove '#'

    match hex_part.len() {
        3 => { // #RGB (e.g., #123)
            let r = hex_part.chars().next()?;
            let g = hex_part.chars().nth(1)?;
            let b = hex_part.chars().nth(2)?;
            if !r.is_ascii_hexdigit() || !g.is_ascii_hexdigit() || !b.is_ascii_hexdigit() {
                return std::option::Option::None;
            }
            std::option::Option::Some(format!("#{r}{r}{g}{g}{b}{b}"))
        }
        4 => { // #RGBA (e.g., #123f)
            let r = hex_part.chars().next()?;
            let g = hex_part.chars().nth(1)?;
            let b = hex_part.chars().nth(2)?;
            let a = hex_part.chars().nth(3)?; // Alpha char
            if !r.is_ascii_hexdigit() || !g.is_ascii_hexdigit() || !b.is_ascii_hexdigit() || !a.is_ascii_hexdigit() {
                return std::option::Option::None; // All chars must be hex, even if alpha is ignored for output format
            }
            std::option::Option::Some(format!("#{r}{r}{g}{g}{b}{b}"))
        }
        6 => { // #RRGGBB (e.g., #112233)
            if hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
                std::option::Option::Some(std::string::String::from(hex_str))
            } else {
                std::option::Option::None
            }
        }
        8 => { // #RRGGBBAA (e.g., #112233ff)
            let rgb_part = &hex_part[0..6];
            let alpha_part = &hex_part[6..8];
            if rgb_part.chars().all(|c| c.is_ascii_hexdigit()) && alpha_part.chars().all(|c| c.is_ascii_hexdigit()) {
                std::option::Option::Some(format!("#{rgb_part}"))
            } else {
                std::option::Option::None
            }
        }
        _ => std::option::Option::None, // Invalid length
    }
}

/// Extracts CSS color values from HTML content.
///
/// This function searches the `html_content` string for patterns matching CSS colors,
/// including hexadecimal, RGB/RGBA, HSL/HSLA, and named color formats.
/// All found color strings are converted to lowercase and collected into a HashSet
/// to ensure uniqueness.
///
/// # Arguments
///
/// * `html_content`: A string slice representing the HTML content to parse.
///
/// # Returns
///
/// A `std::collections::HashSet<std::string::String>` containing all unique color
/// strings found in the HTML, converted to 6-digit lowercase hex format (e.g., "#rrggbb").
///
/// # Panics
///
/// Panics if the internal regex pattern fails to compile (which should not happen with the static pattern).
pub fn extract_colors_from_html(html_content: &str) -> std::collections::HashSet<std::string::String> {
    // Regex provided in the task. The `(?i)` flag makes the pattern case-insensitive.
    // It captures:
    // 1. Hexadecimal: #RRGGBB, #RGB, #RRGGBBAA, #RGBA
   // 2. RGB/RGBA: rgb(r,g,b), rgba(r,g,b,a)
   // 3. HSL/HSLA: hsl(h,s,l), hsla(h,s,l,a)
   // 4. 16 basic W3C named colors (e.g., black, red, blue), with word boundaries.
   let color_regex_pattern = r"(?i)(#(?:[0-9a-f]{8}|[0-9a-f]{6}|[0-9a-f]{4}|[0-9a-f]{3})|(?:rgb|hsl)a?\(\s*(?:\d{1,3}%?\s*,\s*){2}\d{1,3}%?\s*(?:,\s*\d*\.?\d+%?\s*)?\)|(?:black|silver|gray|white|maroon|red|purple|fuchsia|green|lime|olive|yellow|navy|blue|teal|aqua)\b)";
   
   let re = regex::Regex::new(color_regex_pattern)
       .expect("Failed to compile color extraction regex pattern. This is a bug.");

    let mut found_colors = std::collections::HashSet::new();

    // Iterate over all matches. `captures.get(1)` holds the matched color string
    // because the entire regex pattern is enclosed in a capturing group `(...)` after `(?i)`.
    for captures in re.captures_iter(html_content) {
        if let Some(matched_color_capture) = captures.get(1) {
            let lc_color_str = matched_color_capture.as_str().to_lowercase();
            let hex_option: std::option::Option<std::string::String>;

            if lc_color_str.starts_with('#') {
                hex_option = normalize_hex_input(&lc_color_str);
            } else if lc_color_str.starts_with("rgb") { // covers "rgb" and "rgba"
                hex_option = crate::utils::color_conversions::rgb_string_to_hex::rgb_string_to_hex(&lc_color_str);
            } else if lc_color_str.starts_with("hsl") { // covers "hsl" and "hsla"
                hex_option = crate::utils::color_conversions::hsl_string_to_hex::hsl_string_to_hex(&lc_color_str);
            } else { // Assumed to be a named color (e.g., "red", "blue")
                hex_option = crate::utils::color_conversions::named_to_hex::named_color_to_hex(&lc_color_str);
            }

            if let std::option::Option::Some(hex_color) = hex_option {
                found_colors.insert(hex_color);
            }
        }
    }

    found_colors
}

#[cfg(test)]
mod tests {
    // Helper function to check expected colors against actual results.
    // Expected colors must be 6-digit lowercase hex strings.
    fn check_colors(html_content: &str, expected_colors_slice: &[&str]) {
        let actual_colors = super::extract_colors_from_html(html_content);
        
        let mut expected_colors = std::collections::HashSet::new();
        for color_str in expected_colors_slice {
            expected_colors.insert(std::string::String::from(*color_str)); // Already expected in correct format
        }
        
        std::assert_eq!(actual_colors, expected_colors, "Mismatch in extracted colors for HTML: '{}'", html_content);
    }

    #[test]
    fn test_no_colors() {
        check_colors("<div>Some text without any colors.</div>", &[]);
    }

    #[test]
    fn test_hex_colors() {
        let html = "<p style='color: #FF0000; background: #0f0; border-color: #1234AF;'>Text</p>
                    <div style='color:#1234; background:#12345678;'></div>"; // #1234 is #RGBA, #12345678 is #RRGGBBAA
        check_colors(html, &["#ff0000", "#00ff00", "#1234af", "#112233", "#123456"]);
    }

    #[test]
    fn test_hex_colors_case_insensitivity() {
        let html = "<span style='color: #fF00aA; fill: #ABC;'></span>";
        check_colors(html, &["#ff00aa", "#aabbcc"]);
    }

    #[test]
    fn test_rgb_rgba_colors() {
        let html = "style='color: rgb(255, 0, 0); background: rgba(0,0,255,0.5); fill: RGB( 10%,20%, 30% ); stroke: RGBA(0, 128,0, 1);'";
        // rgb(255,0,0) -> #ff0000
        // rgba(0,0,255,0.5) -> #0000ff
        // rgb(10%,20%,30%): 10% of 255 = 26 (1a), 20% of 255 = 51 (33), 30% of 255 = 77 (4d) -> #1a334d
        // rgba(0,128,0,1) -> #008000
        check_colors(html, &["#ff0000", "#0000ff", "#1a334d", "#008000"]);
    }
    
    #[test]
    fn test_hsl_hsla_colors() {
        let html = "color: hsl(120, 100%, 50%); background: hsla(240, 100%, 50%, 0.8); border: HSL(0,0%,0%); fill: HSLA(0, 0%, 100%, .5)";
        // hsl(120, 100%, 50%) -> #00ff00 (green)
        // hsla(240, 100%, 50%, 0.8) -> #0000ff (blue)
        // hsl(0,0%,0%) -> #000000 (black)
        // hsla(0, 0%, 100%, .5) -> #ffffff (white)
        check_colors(html, &["#00ff00", "#0000ff", "#000000", "#ffffff"]);
    }

    #[test]
    fn test_named_colors() {
        let html = "<div style='color: Red; background: SILVER; border-top-color: teal;'></div>";
        // Red -> #ff0000
        // SILVER -> #c0c0c0
        // teal -> #008080
        check_colors(html, &["#ff0000", "#c0c0c0", "#008080"]);
    }

    #[test]
    fn test_named_colors_word_boundaries() {
        // "black" should match, "blackboard" should not contain "black" as a color.
        // "aqua" should match, "aquarium" should not.
        // "red" should match, but not "reddish"
        let html = "color: black; background: blackboard; fill: aqua; stroke: aquariumred; border: red; outline: reddish;";
        // black -> #000000
        // aqua -> #00ffff
        // red -> #ff0000
        check_colors(html, &["#000000", "#00ffff", "#ff0000"]);
    }

    #[test]
    fn test_mixed_colors_and_duplicates() {
        let html = "color: #FFF; color: White; color: #FFF; background: rgb(0,0,0); border: BLACK;";
        // #FFF -> #ffffff
        // White -> #ffffff
        // rgb(0,0,0) -> #000000
        // BLACK -> #000000
        check_colors(html, &["#ffffff", "#000000"]);
    }

    #[test]
    fn test_colors_in_various_contexts() {
        let html = "<body bgcolor='RED' text='#112233'>
                        <p style='color:#123456; border:1px solid rgba(0,50%,100%,.5)'>Hi</p>
                        <style>.cls { fill: green; stroke: HSL(0, 100%, 50%); background-color: #AbC; }</style>
                        <!-- color: yellow -->
                        <svg><rect fill=\"PURPLE\"/></svg>
                    </body>";
        // RED -> #ff0000
        // #112233 -> #112233
        // #123456 -> #123456
        // rgba(0,50%,100%,.5): R=0, G=50%*255=128, B=100%*255=255 -> #0080ff
        // green -> #008000
        // HSL(0, 100%, 50%) -> #ff0000
        // #AbC -> #aabbcc
        // yellow -> #ffff00
        // PURPLE -> #800080
        check_colors(html, &[
            "#ff0000", "#112233", "#123456", "#0080ff", 
            "#008000", "#aabbcc", "#ffff00", "#800080"
        ]);
    }
    
    #[test]
    fn test_empty_html_input() {
        check_colors("", &[]);
    }

    #[test]
    fn test_html_with_only_whitespace() {
        check_colors("   \n\t   ", &[]);
    }
    
    #[test]
    fn test_color_values_with_extra_spaces() {
        // The regex handles spaces within rgb/hsl functions. Hex and named colors don't typically have internal spaces.
        let html = "color: rgb( 50 , 100 , 150 ); background: hsla( 0, 50% , 50% , 0.2 ); border-color: #1A2B3C ;";
        // rgb(50,100,150) -> #326496
        // hsla(0,50%,50%,0.2) -> H=0, S=0.5, L=0.5 -> #bf4040
        // #1A2B3C -> #1a2b3c
        check_colors(html, &["#326496", "#bf4040", "#1a2b3c"]);
    }
}
