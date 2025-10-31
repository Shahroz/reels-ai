//! Converts a `serde_yaml::Value` into a `serde_json::Value`.
//!
//! This file defines the `yaml_value_to_json_value` function, which is an
//! internal helper for transforming YAML data structures into their JSON
//! equivalents. It recursively handles various YAML types including scalars
//! (null, boolean, number, string), sequences (arrays), and mappings (objects).
//! The function ensures that YAML mapping keys are converted to strings as
//! required by JSON. It uses `anyhow::Result` for error handling.

// No use statements as per rust_guidelines.

pub fn yaml_value_to_json_value(
    yaml_value: serde_yaml::Value,
) -> anyhow::Result<serde_json::Value> {
    match yaml_value {
        serde_yaml::Value::Null => Ok(serde_json::Value::Null),
        serde_yaml::Value::Bool(b) => Ok(serde_json::Value::Bool(b)),
        serde_yaml::Value::Number(n) => {
            if n.is_i64() {
                // .unwrap() is safe due to is_i64() check
                Ok(serde_json::Value::Number(serde_json::Number::from(
                    n.as_i64().unwrap(),
                )))
            } else if n.is_u64() {
                // .unwrap() is safe due to is_u64() check
                Ok(serde_json::Value::Number(serde_json::Number::from(
                    n.as_u64().unwrap(),
                )))
            } else if n.is_f64() {
                // .unwrap() is safe due to is_f64() check
                let f = n.as_f64().unwrap();
                serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .ok_or_else(|| anyhow::anyhow!("Failed to convert f64 to serde_json::Number: {}", f))
            } else {
                Err(anyhow::anyhow!("Unsupported YAML number type: {:?}", n))
            }
        }
        serde_yaml::Value::String(s) => Ok(serde_json::Value::String(s)),
        serde_yaml::Value::Sequence(seq) => {
            let mut json_array: Vec<serde_json::Value> = Vec::new();
            for item in seq {
                json_array.push(yaml_value_to_json_value(item)?);
            }
            Ok(serde_json::Value::Array(json_array))
        }
        serde_yaml::Value::Mapping(mapping) => {
            let mut json_map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
            for (key_yaml, value_yaml) in mapping {
                let key_str: String = match key_yaml {
                    serde_yaml::Value::String(s) => s,
                    serde_yaml::Value::Bool(b) => b.to_string(),
                    serde_yaml::Value::Number(n) => n.to_string(),
                    // Other YAML value types are not directly convertible to JSON string keys.
                    other_key_type => {
                        return Err(anyhow::anyhow!(
                            "YAML map key must be a string, boolean, or number to convert to a JSON map key. Encountered: {:?}",
                            other_key_type
                        ))
                    }
                };
                json_map.insert(key_str, yaml_value_to_json_value(value_yaml)?);
            }
            Ok(serde_json::Value::Object(json_map))
        }
        serde_yaml::Value::Tagged(tagged_value) => {
            // Recursively convert the value part of the tagged value.
            // The tag itself is ignored in this direct conversion to JSON.
           yaml_value_to_json_value(tagged_value.value)
        }
    }
}