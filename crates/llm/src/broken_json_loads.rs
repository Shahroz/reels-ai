use serde_json::{Value, Map};

/// Main entrypoint
pub fn repair_json(input: &str, schema: &Value) -> Result<Value, String> {
    // First try parsing normally
    if let Ok(v) = serde_json::from_str::<Value>(input) {
        return Ok(align_to_schema(&v, schema));
    }

    // Try to interpret as a stringified JSON literal
    if let Ok(inner_str) = serde_json::from_str::<String>(input) {
        return try_fix_inner_json(&inner_str, schema);
    }

    // Fallback: try some string-level fixes
    try_fix_inner_json(input, schema)
}

/// Attempts to fix inner JSON string with some heuristics
fn try_fix_inner_json(input: &str, schema: &Value) -> Result<Value, String> {
    // Try simple unescaping first - replace \" with "
    let unescaped = input.replace("\\\"", "\"");
    if let Ok(v) = serde_json::from_str::<Value>(&unescaped) {
        return Ok(align_to_schema(&v, schema));
    }

    let mut repaired = String::new();
    let chars = input.chars().peekable();
    let mut inside_string = false;
    let mut escape_next = false;

    for c in chars {
        match c {
            '\\' if !escape_next => {
                escape_next = true;
                repaired.push(c);
            }
            '"' if !escape_next => {
                inside_string = !inside_string;
                repaired.push(c);
            }
            '"' if escape_next => {
                repaired.push_str("\\\"");
                escape_next = false;
            }
            _ => {
                escape_next = false;
                repaired.push(c);
            }
        }
    }

    match serde_json::from_str::<Value>(&repaired) {
        Ok(v) => Ok(align_to_schema(&v, schema)),
        Err(e) => Err(format!("Still invalid after repair: {}\nOutput: {}", e, repaired))
    }
}

/// Aligns the recovered JSON to the schema, filling in missing keys as null
fn align_to_schema(value: &Value, schema: &Value) -> Value {
    match (value, schema) {
        (Value::Object(obj), Value::Object(schema_obj)) => {
            let mut out = Map::new();
            if let Some(Value::Object(props)) = schema_obj.get("properties") {
                for (key, sub_schema) in props {
                    if let Some(v) = obj.get(key) {
                        out.insert(key.clone(), align_to_schema(v, sub_schema));
                    } else {
                        out.insert(key.clone(), Value::Null);
                    }
                }
            }
            Value::Object(out)
        },
        (Value::Array(arr), Value::Object(schema_obj)) => {
            if let Some(item_schema) = schema_obj.get("items") {
                Value::Array(arr.iter().map(|v| align_to_schema(v, item_schema)).collect())
            } else {
                value.clone()
            }
        },
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repair_llm_stringified_output() {
        let input = r#"{\"title\": \"Example\", \"tags\": [\"one\", \"two\"]}"#;
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "title": {"type": "string"},
                "tags": {"type": "array", "items": {"type": "string"}}
            }
        });

        let result = repair_json(input, &schema);
        println!("Repaired: {:?}", result);
        assert!(result.is_ok());
        let v = result.unwrap();
        assert_eq!(v["title"], "Example");
    }
}