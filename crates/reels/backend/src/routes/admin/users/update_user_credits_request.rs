//! Request structure for updating a user's credits via admin endpoint.
//!
//! This struct defines the payload for credit updates. The credits value will be set
//! as the new absolute balance for the user's personal organization (not added/subtracted).
//! Used by admin users to manually adjust user credit allocations. All credit changes
//! are logged to both the credit transaction log and the audit trail.
//!
//! Validation: Credits must be between 0 and 1,000,000 inclusive.

/// Request payload for updating user credits
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateUserCreditsRequest {
    /// The new credit balance for the user's personal organization (0-1,000,000)
    #[serde(deserialize_with = "validate_credits")]
    #[schema(example = "1000", minimum = 0, maximum = 1000000)]
    pub credits: i32,
}

/// Custom deserializer to validate credits are within allowed range
fn validate_credits<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let credits = <i32 as serde::Deserialize>::deserialize(deserializer)?;
    
    if credits < 0 {
        return Err(serde::de::Error::custom(
            "Credits cannot be negative"
        ));
    }
    
    if credits > 1_000_000 {
        return Err(serde::de::Error::custom(
            "Credits cannot exceed 1,000,000"
        ));
    }
    
    Ok(credits)
}

