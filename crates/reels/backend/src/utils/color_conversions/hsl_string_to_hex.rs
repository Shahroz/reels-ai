//! Parses HSL/HSLA color strings and converts them to a 6-digit hexadecimal format.
//!
//! This module provides `hsl_string_to_hex`, a function that takes an HSL or HSLA
//! color string (e.g., "hsl(120, 100%, 50%)", "hsla(0, 0%, 0%, 0.5)"). It parses
//! Hue (0-360), Saturation (0-100%), and Lightness (0-100%) values.
//! The HSL values are then converted to RGB, and finally formatted as a lowercase
//! 6-digit hexadecimal string (e.g., "#00ff00"). The alpha component in HSLA
//! strings is ignored. Invalid formats or out-of-range HSL values result in `None`.
//! Adheres to project Rust coding standards, including no `use` statements and in-file tests.
//!
//! Revision History
//! - 2025-05-22T16:54:49Z @AI: Initial creation of the file.

// Helper to parse the Hue component (0-360 degrees)
fn parse_hue_component(hue_str: &str) -> std::option::Option<f64> {
    match hue_str.trim().parse::<f64>() {
        std::result::Result::Ok(h) if (0.0..=360.0).contains(&h) => std::option::Option::Some(h),
        _ => std::option::Option::None,
    }
}

// Helper to parse Saturation or Lightness components (0-100%)
// Returns normalized value (0.0-1.0)
fn parse_sl_component(sl_str: &str) -> std::option::Option<f64> {
    let trimmed_sl = sl_str.trim();
    if !trimmed_sl.ends_with('%') {
        return std::option::Option::None;
    }
    match trimmed_sl.trim_end_matches('%').parse::<f64>() {
        std::result::Result::Ok(val) if (0.0..=100.0).contains(&val) => {
            std::option::Option::Some(val / 100.0)
        }
        _ => std::option::Option::None,
    }
}

// Helper function: HSL to RGB conversion.
// h: hue (0-360), s: saturation (0-1), l: lightness (0-1)
fn hsl_to_rgb(h_deg: f64, s_norm: f64, l_norm: f64) -> (u8, u8, u8) {
    // Nested helper: Converts a hue component to an RGB value part.
    // p, q are temporary variables from HSL conversion formula.
    // t is derived from hue (e.g., h_norm + 1.0/3.0 for red).
    fn hue_to_rgb_component(p: f64, q: f64, mut t: f64) -> f64 {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 { // Note: Can also use t >= 1.0 and adjust conditions if t can be exactly 1.0
            t -= 1.0;
        }

        if t < 1.0 / 6.0 {
            return p + (q - p) * 6.0 * t;
        }
        if t < 1.0 / 2.0 {
            return q;
        }
        if t < 2.0 / 3.0 {
            return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
        }
        p
    }

    if s_norm == 0.0 {
        // Achromatic (gray)
        let gray_val = (l_norm * 255.0).round() as u8;
        (gray_val, gray_val, gray_val)
    } else {
        let q = if l_norm < 0.5 {
            l_norm * (1.0 + s_norm)
        } else {
            l_norm + s_norm - l_norm * s_norm
        };
        let p = 2.0 * l_norm - q;

        let h_norm = h_deg / 360.0; // Normalize hue to 0-1 range

        let r_prime = hue_to_rgb_component(p, q, h_norm + 1.0 / 3.0);
        let g_prime = hue_to_rgb_component(p, q, h_norm);
        let b_prime = hue_to_rgb_component(p, q, h_norm - 1.0 / 3.0);

        let r = (r_prime * 255.0).round() as u8;
        let g = (g_prime * 255.0).round() as u8;
        let b = (b_prime * 255.0).round() as u8;
        (r, g, b)
    }
}

pub fn hsl_string_to_hex(hsl_string: &str) -> std::option::Option<std::string::String> {
    // Regex to capture H, S, L components from "hsl(...)" or "hsla(...)" strings.
    // It expects lowercase "hsl" or "hsla".
    // Components are captured by `([^,]+)`. Alpha component is matched but not captured for use.
    let re = match regex::Regex::new(
        r"^(?:hsl|hsla)\s*\(\s*([^,]+)\s*,\s*([^,]+)\s*,\s*([^,]+)\s*(?:,\s*[^)]+)?\s*\)$"
    ) {
        std::result::Result::Ok(r) => r,
        std::result::Result::Err(_) => return std::option::Option::None, // Regex compilation error
    };

    match re.captures(hsl_string.trim().to_lowercase().as_str()) {
        std::option::Option::Some(caps) => {
            let h_str = caps.get(1).map_or("", |m| m.as_str());
            let s_str = caps.get(2).map_or("", |m| m.as_str());
            let l_str = caps.get(3).map_or("", |m| m.as_str());

            match (
                parse_hue_component(h_str),
                parse_sl_component(s_str),
                parse_sl_component(l_str),
            ) {
                (std::option::Option::Some(h), std::option::Option::Some(s_norm), std::option::Option::Some(l_norm)) => {
                    let (r, g, b) = hsl_to_rgb(h, s_norm, l_norm);
                    std::option::Option::Some(format!("#{r:02x}{g:02x}{b:02x}"))
                }
                _ => std::option::Option::None, // Parsing or validation of H, S, or L failed
            }
        }
        std::option::Option::None => std::option::Option::None, // String does not match HSL/HSLA format
    }
}

#[cfg(test)]
mod tests {
    // Accessing the function under test via `super::`.
    // Fully qualified paths for other items.

    #[test]
    fn test_primary_colors() {
        // Red: hsl(0, 100%, 50%)
        std::assert_eq!(
            super::hsl_string_to_hex("hsl(0, 100%, 50%)"),
            std::option::Option::Some(std::string::String::from("#ff0000"))
        );
        // Green: hsl(120, 100%, 50%)
        std::assert_eq!(
            super::hsl_string_to_hex("hsl(120, 100%, 50%)"),
            std::option::Option::Some(std::string::String::from("#00ff00"))
        );
        // Blue: hsl(240, 100%, 50%)
        std::assert_eq!(
            super::hsl_string_to_hex("hsl(240, 100%, 50%)"),
            std::option::Option::Some(std::string::String::from("#0000ff"))
        );
    }

    #[test]
    fn test_secondary_colors() {
        // Yellow: hsl(60, 100%, 50%)
        std::assert_eq!(
            super::hsl_string_to_hex("hsl(60, 100%, 50%)"),
            std::option::Option::Some(std::string::String::from("#ffff00"))
        );
        // Cyan: hsl(180, 100%, 50%)
        std::assert_eq!(
            super::hsl_string_to_hex("hsl(180, 100%, 50%)"),
            std::option::Option::Some(std::string::String::from("#00ffff"))
        );
        // Magenta: hsl(300, 100%, 50%)
        std::assert_eq!(
            super::hsl_string_to_hex("hsl(300, 100%, 50%)"),
            std::option::Option::Some(std::string::String::from("#ff00ff"))
        );
    }

    #[test]
    fn test_grayscales() {
        // White: hsl(0, 0%, 100%) or any H, S=0, L=100
        std::assert_eq!(
            super::hsl_string_to_hex("hsl(0, 0%, 100%)"),
            std::option::Option::Some(std::string::String::from("#ffffff"))
        );
        std::assert_eq!( // Test with non-zero hue
            super::hsl_string_to_hex("hsl(180, 0%, 100%)"),
            std::option::Option::Some(std::string::String::from("#ffffff"))
        );
        // Gray: hsl(0, 0%, 50%)
        std::assert_eq!(
            super::hsl_string_to_hex("hsl(0, 0%, 50%)"),
            std::option::Option::Some(std::string::String::from("#808080"))
        );
        // Black: hsl(0, 0%, 0%)
        std::assert_eq!(
            super::hsl_string_to_hex("hsl(0, 0%, 0%)"),
            std::option::Option::Some(std::string::String::from("#000000"))
        );
        // Light Gray: hsl(0, 0%, 75%) -> #bfbfbf (0.75*255 = 191.25 -> 191)
        std::assert_eq!(
            super::hsl_string_to_hex("hsl(0, 0%, 75%)"),
            std::option::Option::Some(std::string::String::from("#bfbfbf"))
        );
    }

    #[test]
    fn test_hsla_ignoring_alpha() {
        // Red with alpha: hsla(0, 100%, 50%, 0.5)
        std::assert_eq!(
            super::hsl_string_to_hex("hsla(0, 100%, 50%, 0.5)"),
            std::option::Option::Some(std::string::String::from("#ff0000"))
        );
        std::assert_eq!( // Test with alpha as percentage
            super::hsl_string_to_hex("hsla(120, 100%, 50%, 80%)"),
            std::option::Option::Some(std::string::String::from("#00ff00"))
        );
         std::assert_eq!( // Test with alpha as integer
            super::hsl_string_to_hex("hsla(240, 100%, 50%, 1)"),
            std::option::Option::Some(std::string::String::from("#0000ff"))
        );
    }

    #[test]
    fn test_specific_hsl_values() {
        // Orange: hsl(30, 100%, 50%) -> #ff8000
        std::assert_eq!(
            super::hsl_string_to_hex("hsl(30, 100%, 50%)"),
            std::option::Option::Some(std::string::String::from("#ff8000"))
        );
        // Light Sky Blue: hsl(200, 70%, 60%) -> #66a3e0 (approx)
        // H=200, S=0.7, L=0.6
        // q = 0.6 + 0.7 - 0.6 * 0.7 = 1.3 - 0.42 = 0.88
        // p = 2 * 0.6 - 0.88 = 1.2 - 0.88 = 0.32
        // h_norm = 200/360 = 5/9 = 0.555...
        // R: t = 5/9 + 1/3 = 5/9 + 3/9 = 8/9 = 0.888... (> 2/3). R_prime = p = 0.32. R_u8 = (0.32*255).round() = 81.6 -> 82 (52)
        // G: t = 5/9 = 0.555... (between 1/2 and 2/3). G_prime = p + (q-p)*(2/3-t)*6 = 0.32 + (0.56)*(2/3 - 5/9)*6 = 0.32 + 0.56*(6/9-5/9)*6 = 0.32 + 0.56*(1/9)*6 = 0.32 + 0.56 * 2/3 = 0.32 + 0.3733... = 0.6933... G_u8 = (0.6933*255).round() = 176.79 -> 177 (b1)
        // B: t = 5/9 - 1/3 = 5/9 - 3/9 = 2/9 = 0.222... (between 1/6 and 1/2). B_prime = q = 0.88. B_u8 = (0.88*255).round() = 224.4 -> 224 (e0)
        // Expected: #52b1e0
        std::assert_eq!(
            super::hsl_string_to_hex("hsl(200, 70%, 60%)"),
            std::option::Option::Some(std::string::String::from("#52b1e0"))
        );
        // Test value from thought process: hsl(30, 60%, 90%) -> #f5e6d6
        std::assert_eq!(
            super::hsl_string_to_hex("hsl(30, 60%, 90%)"),
            std::option::Option::Some(std::string::String::from("#f5e6d6"))
        );
        // Test hue 360 (should be same as 0)
        std::assert_eq!(
            super::hsl_string_to_hex("hsl(360, 100%, 50%)"),
            std::option::Option::Some(std::string::String::from("#ff0000"))
        );
    }

    #[test]
    fn test_whitespace_and_case_handling() {
        std::assert_eq!(
            super::hsl_string_to_hex("  HSL( 0 ,  100% ,  50% )  "), // Mixed case, spaces
            std::option::Option::Some(std::string::String::from("#ff0000"))
        );
        std::assert_eq!(
            super::hsl_string_to_hex("hsla( 120,100%,50%,0.8 )"), // No spaces after comma
            std::option::Option::Some(std::string::String::from("#00ff00"))
        );
    }
    
    #[test]
    fn test_decimal_values_for_h_s_l() {
        // Hue with decimal
        std::assert_eq!(
            super::hsl_string_to_hex("hsl(0.0, 100%, 50%)"),
            std::option::Option::Some(std::string::String::from("#ff0000"))
        );
        // Saturation with decimal
        std::assert_eq!(
            super::hsl_string_to_hex("hsl(180, 50.5%, 50%)"), // S=0.505, L=0.5, H=180 (Cyanish)
            // H=180, S=0.505, L=0.5
            // q = 0.5 * (1 + 0.505) = 0.5 * 1.505 = 0.7525
            // p = 2 * 0.5 - 0.7525 = 1 - 0.7525 = 0.2475
            // h_norm = 0.5
            // R: t = 0.5 + 1/3 = 0.833... (>2/3). R_prime = p = 0.2475. R_u8 = (0.2475*255).round() = 63.11 -> 63 (3f)
            // G: t = 0.5 (=1/2). G_prime = q = 0.7525. G_u8 = (0.7525*255).round() = 191.88 -> 192 (c0)
            // B: t = 0.5 - 1/3 = 1/6. B_prime = p + (q-p)*6*t = 0.2475 + (0.505)*6*(1/6) = 0.2475 + 0.505 = 0.7525. B_u8 = 192 (c0)
            std::option::Option::Some(std::string::String::from("#3fc0c0"))
        );
        // Lightness with decimal
        std::assert_eq!(
            super::hsl_string_to_hex("hsl(240, 100%, 25.5%)"), // L=0.255 (Dark Blue)
            // H=240, S=1, L=0.255
            // q = 0.255 * (1+1) = 0.51
            // p = 2 * 0.255 - 0.51 = 0.51 - 0.51 = 0
            // h_norm = 240/360 = 2/3
            // R: t = 2/3 + 1/3 = 1. Adjusted t=0. R_prime = p = 0. R_u8 = 0
            // G: t = 2/3. G_prime = p = 0. R_u8 = 0
            // B: t = 2/3 - 1/3 = 1/3. B_prime = q = 0.51. B_u8 = (0.51*255).round() = 130.05 -> 130 (82)
            std::option::Option::Some(std::string::String::from("#000082"))
        );
    }

    #[test]
    fn test_invalid_format() {
        std::assert_eq!(super::hsl_string_to_hex("hsl(0, 100%)"), std::option::Option::None); // Too few components
        std::assert_eq!(super::hsl_string_to_hex("hls(0,0%,0%)"), std::option::Option::None); // Incorrect prefix
        std::assert_eq!(super::hsl_string_to_hex("hsl(0,0,0)"), std::option::Option::None); // Missing % for S and L
        std::assert_eq!(super::hsl_string_to_hex("hsl(0%,0%,0%)"), std::option::Option::None); // % for H
        std::assert_eq!(super::hsl_string_to_hex("hsl(foo,bar,baz)"), std::option::Option::None); // Non-numeric
        std::assert_eq!(super::hsl_string_to_hex("hsl(0,100%,50%"), std::option::Option::None); // Missing closing paren
        std::assert_eq!(super::hsl_string_to_hex("hsl()"), std::option::Option::None); // Empty
        std::assert_eq!(super::hsl_string_to_hex(""), std::option::Option::None); // Empty string
    }

    #[test]
    fn test_out_of_range_values() {
        // Hue
        std::assert_eq!(super::hsl_string_to_hex("hsl(361, 100%, 50%)"), std::option::Option::None);
        std::assert_eq!(super::hsl_string_to_hex("hsl(-1, 100%, 50%)"), std::option::Option::None);
        // Saturation
        std::assert_eq!(super::hsl_string_to_hex("hsl(0, 101%, 50%)"), std::option::Option::None);
        std::assert_eq!(super::hsl_string_to_hex("hsl(0, -1%, 50%)"), std::option::Option::None);
        // Lightness
        std::assert_eq!(super::hsl_string_to_hex("hsl(0, 100%, 101%)"), std::option::Option::None);
        std::assert_eq!(super::hsl_string_to_hex("hsl(0, 100%, -1%)"), std::option::Option::None);
    }
}