//! Defines the `Tool` enum and supporting wrapper structs for representing tools the Gemini model can use.
//!
//! The `Tool` enum uses `#[serde(untagged)]` and wrapper structs (`GoogleSearchToolWrapper`,
//! `FunctionDeclarationsToolWrapper`) to ensure correct JSON serialization for the Gemini API.
//! Each tool configuration (e.g., Google Search, Function Declarations) is represented by one of these wrappers,
//! which then becomes a variant of the `Tool` enum. This structure ensures the `tools` array in
//! API requests contains objects with the correct top-level keys like `googleSearch` or `function_declarations`.
//! Adheres to coding guidelines, using fully qualified paths and file-level documentation.

/// Wrapper for Google Search tool configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct GoogleSearchToolWrapper {
    /// Contains the Google Search configuration (currently an empty struct).
    #[serde(rename = "google_search")]
    pub google_search: crate::vendors::gemini::google_search::GoogleSearch,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct URLContext {} // Empty struct as per docs

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct URLContextToolWrapper {
    /// Contains the Google Search configuration (currently an empty struct).
    #[serde(rename = "url_context")]
    pub url_context: URLContext,
}

/// Wrapper for Function Declarations tool configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FunctionDeclarationsToolWrapper {
    /// A list of function declarations.
    #[serde(rename = "function_declarations")] // Ensures snake_case key as per API
    pub function_declarations: Vec<crate::vendors::gemini::function_declaration::FunctionDeclaration>,
}

/// Represents a tool that the Gemini model can use.
///
/// This enum is `#[serde(untagged)]` to allow its variants (which are wrapper structs)
/// to serialize directly into the format expected by the Gemini API for elements
/// within the `tools` array.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Tool {
    /// Represents a Google Search tool.
    URLContext(URLContextToolWrapper),
    /// Represents a Google Search tool.
    GoogleSearch(GoogleSearchToolWrapper),
    /// Represents a set of function declarations for function calling.
    FunctionDeclarations(FunctionDeclarationsToolWrapper),
}

#[cfg(test)]
mod tests {
    // Use super::Tool for the enum under test.
    // Use super::GoogleSearchToolWrapper and super::FunctionDeclarationsToolWrapper for wrappers.
    // Fully qualified paths for other crate types.
    // String, Vec, Option are used directly as they are typically in prelude. `serde_json` items require FQN.

    #[test]
    fn test_google_search_tool_wrapper_serialization() {
        let google_search_wrapper = super::GoogleSearchToolWrapper {
            google_search: crate::vendors::gemini::google_search::GoogleSearch {},
        };
        let tool_variant = super::Tool::GoogleSearch(google_search_wrapper);

        let json_str = serde_json::to_string(&tool_variant).unwrap();
        assert_eq!(json_str, r#"{"google_search":{}}"#);

        let deserialized_tool: super::Tool = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized_tool, tool_variant);
    }

    #[test]
    fn test_function_declarations_tool_wrapper_serialization() {
        let parameters_json = serde_json::json!({
            "type": "OBJECT",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "The city and state, e.g. San Francisco, CA"
                }
            },
            "required": ["location"]
        });

        let func_decl = crate::vendors::gemini::function_declaration::FunctionDeclaration {
            name: String::from("getCurrentWeather"),
            description: String::from("Get the current weather in a given location"),
            parameters: parameters_json,
        };

        let func_declarations_wrapper = super::FunctionDeclarationsToolWrapper {
            function_declarations: vec![func_decl.clone()],
        };
        let tool_variant = super::Tool::FunctionDeclarations(func_declarations_wrapper);

        let json_str = serde_json::to_string(&tool_variant).unwrap();
        // Expected: {"function_declarations":[{"name":"getCurrentWeather",...}]}
        // The key is "function_declarations" (snake_case) due to the wrapper struct's field rename.

        // Deserialize and check fields, as HashMap serialization order can vary
        let deserialized_tool: super::Tool = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized_tool, tool_variant);

        match deserialized_tool {
            super::Tool::FunctionDeclarations(wrapper) => {
                assert_eq!(wrapper.function_declarations.len(), 1);
                let deserialized_func_decl = &wrapper.function_declarations[0];
                assert_eq!(deserialized_func_decl.name, "getCurrentWeather");
                assert_eq!(
                    deserialized_func_decl.description,
                    "Get the current weather in a given location"
                );
                assert_eq!(deserialized_func_decl.parameters["type"], "OBJECT");
                assert_eq!(deserialized_func_decl.parameters["properties"]["location"]["type"], "string");
                assert_eq!(deserialized_func_decl.parameters["properties"]["location"]["description"], "The city and state, e.g. San Francisco, CA");
                assert_eq!(deserialized_func_decl.parameters["required"][0], "location");
            }
            _ => panic!("Deserialized into wrong variant, expected FunctionDeclarations"),
        }
    }

    #[test]
    fn test_function_declarations_tool_deserialization_from_api_format() {
        let json_input = r#"{
            "function_declarations": [
                {
                    "name": "findFlights",
                    "description": "Finds flights for a given origin and destination",
                    "parameters": {
                        "type": "OBJECT",
                        "properties": {
                            "origin": { "type": "string", "description": "Departure airport" },
                            "destination": { "type": "string", "description": "Arrival airport" }
                        },
                        "required": ["origin", "destination"]
                    }
                }
            ]
        }"#;
        let deserialized_tool: super::Tool = serde_json::from_str(json_input).unwrap();
        match deserialized_tool {
            super::Tool::FunctionDeclarations(wrapper) => {
                assert_eq!(wrapper.function_declarations.len(), 1);
                assert_eq!(wrapper.function_declarations[0].name, "findFlights");
                assert_eq!(
                    wrapper.function_declarations[0].parameters["properties"]["origin"]["type"].as_str().unwrap(),
                    "string"
                );
            }
           _ => panic!("Deserialized static JSON into wrong variant, expected FunctionDeclarations"),
       }
   }
}
