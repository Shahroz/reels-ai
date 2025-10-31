//! User account status enumeration.
//!
//! Defines the possible states of a user account: active, trial, expired, and cancelled.
//! Used for validating status changes and ensuring type safety throughout the application.
//! All status values are lowercase strings in the database.

/// Enum for user account statuses
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    /// User account is active and in good standing
    Active,
    /// User is in trial period
    Trial,
    /// User's trial or subscription has expired
    Expired,
    /// User account has been cancelled
    Cancelled,
}

impl std::fmt::Display for UserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserStatus::Active => write!(f, "active"),
            UserStatus::Trial => write!(f, "trial"),
            UserStatus::Expired => write!(f, "expired"),
            UserStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for UserStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(UserStatus::Active),
            "trial" => Ok(UserStatus::Trial),
            "expired" => Ok(UserStatus::Expired),
            "cancelled" => Ok(UserStatus::Cancelled),
            _ => Err(format!("'{}' is not a valid user status", s)),
        }
    }
}

