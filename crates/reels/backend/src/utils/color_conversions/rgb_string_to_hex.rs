//! Parses RGB/RGBA color strings and converts them to a 6-digit hexadecimal format.
//!
//! This function handles 'rgb(R,G,B)' and 'rgba(R,G,B,A)' formats where R, G, B
//! are color components. Numerical components can be integers (0-255) or
//! percentages (0%-100%; e.g., 50% is 128). The alpha component in RGBA strings
//! is ignored. Invalid formats or out-of-range values result in `None`.
//! Adheres to project Rust coding standards.
//!
//! Revision History
//! - 2025-05-22T16:51:45Z @AI: Initial creation of the file.

pub fn rgb_string_to_hex(rgb_string: &str) -> std::option::Option<std::string::String> {
    // Helper function to parse a single color component (R, G, or B).
    // Input `comp_str` is a string slice like "128" or "50%".
    // It's expected to be already trimmed of surrounding whitespace by the regex capture.
    fn parse_component(comp_str: &str) -> std::option::Option<u8> {
        if comp_str.ends_with('%') {
            // Percentage value, e.g., "50%"
            match comp_str.trim_end_matches('%').parse::<f64>() {
                std::result::Result::Ok(val) => {
                    if !(0.0..=100.0).contains(&val) { // Percentage must be 0-100
                        return std::option::Option::None;
                    }
                    // Convert percentage to 0-255 scale, rounding to nearest int (0.5 rounds up)
                    let converted = (val / 100.0 * 255.0).round();
                    // Ensure the rounded value is within u8 range
                    if !(0.0..=255.0).contains(&converted) { // Should be redundant if % was 0-100
                         return std::option::Option::None;
                    }
                    std::option::Option::Some(converted as u8)
                }
                std::result::Result::Err(_) => std::option::Option::None, // Failed to parse number part
            }
        } else {
            // Absolute value, e.g., "128"
            match comp_str.parse::<u16>() { // Parse as u16 to catch values > 255
                std::result::Result::Ok(val) => {
                    if val > 255 { // Absolute value must be 0-255
                        return std::option::Option::None;
                    }
                    std::option::Option::Some(val as u8)
                }
                std::result::Result::Err(_) => std::option::Option::None, // Failed to parse number
            }
        }
    }

    let rgb_string_trimmed = rgb_string.trim();

    // Regex for "rgb(R,G,B)"
    // Note: For production, these regexes should ideally be compiled once (e.g., using once_cell::sync::Lazy).
    // For this fix, direct compilation is used to match the existing style in this function.
    // A regex compilation error here indicates a bug in the regex string itself.
    let re_rgb = match regex::Regex::new(r"^rgb\s*\(\s*([^,\s]+)\s*,\s*([^,\s]+)\s*,\s*([^,\s]+)\s*\)$") {
        std::result::Result::Ok(r) => r,
        std::result::Result::Err(_) => return std::option::Option::None, // Or panic, as invalid static regex is a programming error.
    };

    if let Some(caps) = re_rgb.captures(rgb_string_trimmed) {
        let r_str = caps.get(1).map_or("", |m| m.as_str());
        let g_str = caps.get(2).map_or("", |m| m.as_str());
        let b_str = caps.get(3).map_or("", |m| m.as_str());
        
        return match (parse_component(r_str), parse_component(g_str), parse_component(b_str)) {
            (std::option::Option::Some(r_val), std::option::Option::Some(g_val), std::option::Option::Some(b_val)) => {
                std::option::Option::Some(format!("#{r_val:02x}{g_val:02x}{b_val:02x}"))
            }
            _ => std::option::Option::None,
        };
    }

    // Regex for "rgba(R,G,B[,A])" - Alpha component is ignored.
    // It expects lowercase "rgba".
    // Components are captured by `([^,\s]+)`.
    // Alpha component, if present, is matched by `(?:,\s*[^)]+)?` but its value is not used.
    let re_rgba = match regex::Regex::new(r"^rgba\s*\(\s*([^,\s]+)\s*,\s*([^,\s]+)\s*,\s*([^,\s]+)\s*(?:,\s*[^)]+)?\s*\)$") {
        std::result::Result::Ok(r) => r,
        std::result::Result::Err(_) => return std::option::Option::None, // Or panic.
    };

    if let Some(caps) = re_rgba.captures(rgb_string_trimmed) {
        // Capture groups start at index 1.
        // .map_or provides a default empty string if capture group is somehow missing (should not happen with this regex structure).
        let r_str = caps.get(1).map_or("", |m| m.as_str());
        let g_str = caps.get(2).map_or("", |m| m.as_str());
        let b_str = caps.get(3).map_or("", |m| m.as_str());

        // Alpha component (if it existed as a 4th capture group) would be ignored as per requirements.
        // The regex `(?:,\s*[^)]+)?` handles the structure of the optional alpha part.
        return match (parse_component(r_str), parse_component(g_str), parse_component(b_str)) {
            (std::option::Option::Some(r_val), std::option::Option::Some(g_val), std::option::Option::Some(b_val)) => {
                std::option::Option::Some(format!("#{r_val:02x}{g_val:02x}{b_val:02x}"))
            }
            _ => std::option::Option::None, // One or more components failed to parse or were invalid
        };
    }
    
    std::option::Option::None // String does not match RGB or RGBA format
}

#[cfg(test)]
mod tests {
    // Accessing the function under test via `super::`.
    // Fully qualified paths for other items (e.g., `std::option::Option`, `std::string::String`).

    #[test]
    fn test_valid_integer_rgb() {
        std::assert_eq!(
            super::rgb_string_to_hex("rgb(255,0,0)"),
            std::option::Option::Some(std::string::String::from("#ff0000"))
        );
        std::assert_eq!(
            super::rgb_string_to_hex("rgb(0, 255, 0)"), // With spaces
            std::option::Option::Some(std::string::String::from("#00ff00"))
        );
        std::assert_eq!(
            super::rgb_string_to_hex("rgb(0,0,255)"),
            std::option::Option::Some(std::string::String::from("#0000ff"))
        );
        std::assert_eq!(
            super::rgb_string_to_hex("rgb(10, 20, 30)"),
            std::option::Option::Some(std::string::String::from("#0a141e"))
        );
    }

    #[test]
    fn test_valid_percentage_rgb() {
        std::assert_eq!(
            super::rgb_string_to_hex("rgb(100%,0%,0%)"),
            std::option::Option::Some(std::string::String::from("#ff0000"))
        );
        std::assert_eq!(
            super::rgb_string_to_hex("rgb(0%, 100%, 0%)"), // With spaces
            std::option::Option::Some(std::string::String::from("#00ff00"))
        );
        std::assert_eq!(
            super::rgb_string_to_hex("rgb(50%,50%,50%)"), // 127.5 rounds to 128
            std::option::Option::Some(std::string::String::from("#808080"))
        );
        // 20% of 255 = 51, 40% of 255 = 102, 60% of 255 = 153
        std::assert_eq!(
            super::rgb_string_to_hex("rgb(20%,40%,60%)"),
            std::option::Option::Some(std::string::String::from("#336699"))
        );
    }

    #[test]
    fn test_mixed_integer_percentage_rgb() {
        // 50% of 255 = 128
        std::assert_eq!(
            super::rgb_string_to_hex("rgb(255,50%,0)"),
            std::option::Option::Some(std::string::String::from("#ff8000"))
        );
        // 10% -> (0.1*255).round() = 25.5.round() = 26 (#1a)
        // 60% -> (0.6*255).round() = 153.0.round() = 153 (#99)
        std::assert_eq!(
            super::rgb_string_to_hex("rgb(10%,128,60%)"),
            std::option::Option::Some(std::string::String::from("#1a8099"))
        );
    }

    #[test]
    fn test_invalid_format() {
        std::assert_eq!(super::rgb_string_to_hex("rgb(255,0)"), std::option::Option::None); // Too few components
        std::assert_eq!(super::rgb_string_to_hex("rbg(1,2,3)"), std::option::Option::None); // Incorrect prefix
        std::assert_eq!(super::rgb_string_to_hex("rgb(1,2,3,4,5)"), std::option::Option::None); // Invalid structure for alpha
        std::assert_eq!(super::rgb_string_to_hex("rgb(1,2,3"), std::option::Option::None); // Missing closing parenthesis
        std::assert_eq!(super::rgb_string_to_hex("rgb(1 2 3)"), std::option::Option::None); // Missing commas
        std::assert_eq!(super::rgb_string_to_hex("rgb(foo,bar,baz)"), std::option::Option::None); // Non-numeric components
        std::assert_eq!(super::rgb_string_to_hex("rgb(10,20,%)"), std::option::Option::None); // Invalid percentage
        std::assert_eq!(super::rgb_string_to_hex(""), std::option::Option::None); // Empty string
        std::assert_eq!(super::rgb_string_to_hex("rgb()"), std::option::Option::None); // Empty components
    }

    #[test]
    fn test_out_of_range_values() {
        std::assert_eq!(super::rgb_string_to_hex("rgb(256,0,0)"), std::option::Option::None); // R > 255
        std::assert_eq!(super::rgb_string_to_hex("rgb(0,256,0)"), std::option::Option::None); // G > 255
        std::assert_eq!(super::rgb_string_to_hex("rgb(0,0,256)"), std::option::Option::None); // B > 255
        std::assert_eq!(super::rgb_string_to_hex("rgb(-1,0,0)"), std::option::Option::None); // R < 0
        std::assert_eq!(super::rgb_string_to_hex("rgb(101%,0%,0%)"), std::option::Option::None); // R% > 100%
        std::assert_eq!(super::rgb_string_to_hex("rgb(0%,-10%,0%)"), std::option::Option::None); // G% < 0%
    }

    #[test]
    fn test_whitespace_handling() {
        // R = 10 -> 0a
        // G = 20% -> (0.2 * 255).round() = 51.0.round() = 51 -> 33
        // B = 30 -> 1e
        // Expected: #0a331e
        std::assert_eq!(
            super::rgb_string_to_hex("  rgb  (  10  ,  20%  ,  30  )  "), // Lots of whitespace
            std::option::Option::Some(std::string::String::from("#0a331e"))
        );
    }
    
    #[test]
    fn test_specific_percentage_rounding() {
        // 12.3% of 255 = 31.365, rounds to 31 (#1f)
        std::assert_eq!(
            super::rgb_string_to_hex("rgb(12.3%,0,0)"),
            std::option::Option::Some(std::string::String::from("#1f0000"))
        );
        // 12.5% of 255 = 31.875, rounds to 32 (#20)
        std::assert_eq!(
            super::rgb_string_to_hex("rgb(12.5%,0,0)"),
            std::option::Option::Some(std::string::String::from("#200000"))
        );
        // 12.8% of 255 = 32.64, rounds to 33 (#21)
         std::assert_eq!(
            super::rgb_string_to_hex("rgb(12.8%,0,0)"),
            std::option::Option::Some(std::string::String::from("#210000"))
        );
        // 0.5% of 255 = 1.275, rounds to 1
        std::assert_eq!(super::rgb_string_to_hex("rgb(0.5%,0,0)"), std::option::Option::Some(std::string::String::from("#010000")));
        // 0.4% of 255 = 1.02, rounds to 1
        std::assert_eq!(super::rgb_string_to_hex("rgb(0.4%,0,0)"), std::option::Option::Some(std::string::String::from("#010000")));

    }
}
