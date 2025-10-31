//! Converts a `toml::Value` into a `serde_json::Value`.
//!
//! This file defines the `toml_value_to_json_value` function, an internal helper
//! for transforming TOML data structures into their JSON equivalents.
//! It leverages `serde_json` to perform the conversion by first serializing
//! the `toml::Value` to a JSON string, and then parsing that string back
//! into a `serde_json::Value`. This assumes that `toml::Value` implements
//! `serde::Serialize` in a way that `serde_json::to_string` can process.
//! This function uses `anyhow::Result` for error handling.

// No `use` statements as per rust_guidelines.
// All paths should be fully qualified if not in prelude.

// Function is `pub` so it can be called from the parent module (`mod.rs`).
// The module itself is kept private to the `llm_typed_unified` scope via `mod toml_value_to_json_value;`
// in `mod.rs` (i.e., not `pub mod`).
pub fn toml_value_to_json_value(
    toml_value: &toml::Value,
) -> anyhow::Result<serde_json::Value> {
    // Serialize the toml::Value directly to a JSON string.
    // This assumes toml::Value implements serde::Serialize.
    let json_string = serde_json::to_string(toml_value)
        .map_err(|e| anyhow::anyhow!("Failed to serialize toml::Value to JSON string: {}", e))?;

    // Parse the JSON string back into a serde_json::Value.
    let json_value: serde_json::Value = serde_json::from_str(&json_string)
        .map_err(|e| anyhow::anyhow!("Failed to parse JSON string from TOML into serde_json::Value: {}", e))?;

    Ok(json_value)
}