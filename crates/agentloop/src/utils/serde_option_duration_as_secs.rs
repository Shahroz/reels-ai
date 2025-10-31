//! Serializes/deserializes `Option<std::time::Duration>` as `Option<u64>` seconds for serde.
//!
//! Handles conversion between Duration and seconds.
//! Used via `#[serde(with = "...")]` attribute.
//! Ensures compatibility with formats expecting seconds.
//! Follows FQN and one-item-per-file guidelines.

//! Revision History
//! - 2025-04-24T14:47:42Z @AI: Initial implementation based on requirement from status_response.rs.

/// Serializes an `Option<Duration>` to `Option<u64>` seconds.
pub fn serialize<S>(duration: &std::option::Option<std::time::Duration>, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match duration {
        Some(d) => serializer.serialize_some(&d.as_secs()),
        None => serializer.serialize_none(),
    }
}

/// Deserializes an `Option<u64>` seconds to `Option<Duration>`.
pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<std::option::Option<std::time::Duration>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let secs: std::option::Option<u64> = serde::Deserialize::deserialize(deserializer)?;
    Ok(secs.map(std::time::Duration::from_secs))
}

// No tests included for this helper module in this iteration.
// Tests would involve checking serialization/deserialization round trips.
