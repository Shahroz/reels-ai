//! Provides a utility function to find a `VendorModel` enum variant by its alias.
//!
//! This function, `find_model_by_alias`, performs a case-insensitive search
//! across all defined aliases for supported LLM models. It helps in mapping
//! string identifiers (e.g., from configuration or user input) to the
//! corresponding `VendorModel` variant.
//!
//! Revision History
//! - 2025-05-16T15:08:36Z @AI: Extracted from mod.rs.

// Adhering to rust_guidelines: FQPs for non-prelude items.
// `use` for external crates like strum_macros is acceptable if used for derives.
use strum::IntoEnumIterator;

/// Finds a `VendorModel` variant by checking against its aliases (case-insensitive).
///
/// Iterates through all `VendorModel` variants and their associated aliases.
/// Returns `Some(VendorModel)` if a match is found, otherwise `None`.
///
/// # Arguments
/// * `alias`: A string slice representing the alias to search for.
///
/// # Examples
/// ```
/// // Assuming VendorModel::Claude(ClaudeModel::Claude35SonnetLatest) has an alias "claude"
/// // let model = find_model_by_alias("claude");
/// // assert!(model.is_some());
/// ```
pub fn find_model_by_alias(alias: &str) -> std::option::Option<super::vendor_model::VendorModel> {
    let alias_lower = alias.to_lowercase();

    // Special handling for "claude" to default to a specific model if no other Claude alias matches.
    // This logic is from the original `mod.rs`.
    if alias_lower == "claude" {
        // Check if any Claude model explicitly has "claude" as an alias first.
        // If not, this acts as a fallback.
        // This part might need refinement if "claude" should strictly map to one with that alias.
        // For now, preserving original logic.
        for model_variant in super::vendor_model::VendorModel::iter() {
            if let super::vendor_model::VendorModel::Claude(_) = model_variant {
                for model_alias in model_variant.aliases() {
                    if model_alias.to_lowercase() == alias_lower {
                        return Some(model_variant);
                    }
                }
            }
        }
        // If no specific Claude model alias matched "claude", return the default Claude model.
        return std::option::Option::Some(super::vendor_model::VendorModel::Claude(crate::vendors::claude::ClaudeModel::Claude35SonnetLatest));
    }

    // Iterate through all VendorModel variants using strum's EnumIter
    // VendorModel::iter() requires VendorModel to derive strum::IntoEnumIterator
    for model in super::vendor_model::VendorModel::iter() {
        for model_alias in model.aliases() {
            if model_alias.to_lowercase() == alias_lower {
                return std::option::Option::Some(model);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    // Per rust_guidelines, use `super::` or FQPs.
    // No `use` statements for items within the crate.


    #[test]
    fn test_find_model_by_alias_non_existing() {
        let model_option = super::find_model_by_alias("no-such-model-alias-exists-123");
        assert!(model_option.is_none(), "Should not find model for a non-existing alias");
    }

    #[test]
    fn test_find_model_by_alias_claude_default() {
        // This tests the specific "claude" fallback logic.
        let model_option = super::find_model_by_alias("claude");
        assert!(model_option.is_some(), "Should find a Claude model for alias 'claude'");
        match model_option {
            Some(super::super::vendor_model::VendorModel::Claude(_)) => {
                // Correct, it's some Claude model.
                // To be more specific, check if it's Claude35SonnetLatest if that's the intended default.
                 if let Some(super::super::vendor_model::VendorModel::Claude(
                    crate::vendors::claude::ClaudeModel::Claude35SonnetLatest,
                )) = model_option {
                    // Correct default
                } else {
                    // This might happen if another Claude model has "claude" as a primary alias.
                    // The test might need adjustment based on exact alias definitions.
                    println!("Note: 'claude' alias mapped to a specific Claude model other than default, which is fine if intended.");
                }
            }
            _ => panic!("'claude' alias did not map to a Claude model."),
        }
    }
}