//! Converts common CSS named colors to their hexadecimal string representation.
//!
//! This module provides a function `named_color_to_hex` that takes a lowercase
//! CSS color name (e.g., "red", "blue") and returns its corresponding 6-digit
//! hexadecimal color code (e.g., "#ff0000") as an `Option<String>`.
//! If the color name is not recognized from a predefined list of 16 common colors,
//! it returns `None`. Adheres to project Rust coding standards.

pub fn named_color_to_hex(name: &str) -> std::option::Option<std::string::String> {
    // Function body must be < 50 LoC. This match statement is well within that.
    // Input 'name' is expected to be lowercase as per instruction.
    match name {
        "black"   => std::option::Option::Some(std::string::String::from("#000000")),
        "silver"  => std::option::Option::Some(std::string::String::from("#c0c0c0")),
        "gray"    => std::option::Option::Some(std::string::String::from("#808080")), // Also known as grey
        "white"   => std::option::Option::Some(std::string::String::from("#ffffff")),
        "maroon"  => std::option::Option::Some(std::string::String::from("#800000")),
        "red"     => std::option::Option::Some(std::string::String::from("#ff0000")),
        "purple"  => std::option::Option::Some(std::string::String::from("#800080")),
        "fuchsia" => std::option::Option::Some(std::string::String::from("#ff00ff")), // Also known as magenta
        "green"   => std::option::Option::Some(std::string::String::from("#008000")),
        "lime"    => std::option::Option::Some(std::string::String::from("#00ff00")),
        "olive"   => std::option::Option::Some(std::string::String::from("#808000")),
        "yellow"  => std::option::Option::Some(std::string::String::from("#ffff00")),
        "navy"    => std::option::Option::Some(std::string::String::from("#000080")),
        "blue"    => std::option::Option::Some(std::string::String::from("#0000ff")),
        "teal"    => std::option::Option::Some(std::string::String::from("#008080")),
        "aqua"    => std::option::Option::Some(std::string::String::from("#00ffff")), // Also known as cyan
        _         => std::option::Option::None, // Handle unknown names
    }
}

#[cfg(test)]
mod tests {
    // Adhering to "No Imports: Use Fully Qualified Paths" even in tests.
    // Accessing the function under test via `super::`.

    #[test]
    fn test_known_color_red() {
        let expected = std::option::Option::Some(std::string::String::from("#ff0000"));
        std::assert_eq!(super::named_color_to_hex("red"), expected);
    }

    #[test]
    fn test_known_color_blue() {
        let expected = std::option::Option::Some(std::string::String::from("#0000ff"));
        std::assert_eq!(super::named_color_to_hex("blue"), expected);
    }

    #[test]
    fn test_known_color_black() {
        let expected = std::option::Option::Some(std::string::String::from("#000000"));
        std::assert_eq!(super::named_color_to_hex("black"), expected);
    }

    #[test]
    fn test_known_color_white() {
        let expected = std::option::Option::Some(std::string::String::from("#ffffff"));
        std::assert_eq!(super::named_color_to_hex("white"), expected);
    }
    
    #[test]
    fn test_known_color_fuchsia() {
        let expected = std::option::Option::Some(std::string::String::from("#ff00ff"));
        std::assert_eq!(super::named_color_to_hex("fuchsia"), expected);
    }

    #[test]
    fn test_unknown_color() {
        // Test for a color name not in the list
        std::assert_eq!(super::named_color_to_hex("chartreuse"), std::option::Option::None);
    }

    #[test]
    fn test_empty_string_input() {
        // Test behavior with an empty string input
        std::assert_eq!(super::named_color_to_hex(""), std::option::Option::None);
    }

    #[test]
    fn test_all_sixteen_specified_colors() {
        // Comprehensive test for all 16 required colors
        let color_map = [
            ("black", "#000000"), ("silver", "#c0c0c0"), ("gray", "#808080"),
            ("white", "#ffffff"), ("maroon", "#800000"), ("red", "#ff0000"),
            ("purple", "#800080"), ("fuchsia", "#ff00ff"), ("green", "#008000"),
            ("lime", "#00ff00"), ("olive", "#808000"), ("yellow", "#ffff00"),
            ("navy", "#000080"), ("blue", "#0000ff"), ("teal", "#008080"),
            ("aqua", "#00ffff"),
        ];

        for (name, hex_code) in color_map.iter() {
            let expected = std::option::Option::Some(std::string::String::from(*hex_code));
            let result = super::named_color_to_hex(name);
            std::assert_eq!(result, expected, "Test failed for color name: {}", name);
        }
    }
}