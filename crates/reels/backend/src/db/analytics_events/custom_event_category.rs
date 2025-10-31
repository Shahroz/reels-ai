//! Custom event category enum for business-specific event classification.
//!
//! Defines categories for custom events: analytics, engagement, conversion, feature usage, other.
//! Stored as string in database for flexibility while maintaining type safety in Rust.
//! Provides conversion methods for database integration and validation.
//! Used to categorize business events beyond automatic middleware tracking.

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CustomEventCategory {
    Analytics,
    Engagement,
    Conversion,
    FeatureUsage,
    Other,
}

impl CustomEventCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            CustomEventCategory::Analytics => "analytics",
            CustomEventCategory::Engagement => "engagement", 
            CustomEventCategory::Conversion => "conversion",
            CustomEventCategory::FeatureUsage => "feature_usage",
            CustomEventCategory::Other => "other",
        }
    }
    
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "analytics" => Ok(CustomEventCategory::Analytics),
            "engagement" => Ok(CustomEventCategory::Engagement),
            "conversion" => Ok(CustomEventCategory::Conversion),
            "feature_usage" => Ok(CustomEventCategory::FeatureUsage),
            "other" => Ok(CustomEventCategory::Other),
            _ => Err(std::format!("Invalid event category: {}", s)),
        }
    }
}

impl std::convert::TryFrom<String> for CustomEventCategory {
    type Error = String;
    
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl std::convert::From<CustomEventCategory> for String {
    fn from(category: CustomEventCategory) -> Self {
        category.as_str().to_string()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_custom_event_category_as_str() {
        assert_eq!(super::CustomEventCategory::Analytics.as_str(), "analytics");
        assert_eq!(super::CustomEventCategory::Engagement.as_str(), "engagement");
        assert_eq!(super::CustomEventCategory::Conversion.as_str(), "conversion");
        assert_eq!(super::CustomEventCategory::FeatureUsage.as_str(), "feature_usage");
        assert_eq!(super::CustomEventCategory::Other.as_str(), "other");
    }

    #[test]
    fn test_custom_event_category_from_str() {
        assert_eq!(
            super::CustomEventCategory::from_str("analytics").unwrap(),
            super::CustomEventCategory::Analytics
        );
        assert_eq!(
            super::CustomEventCategory::from_str("feature_usage").unwrap(),
            super::CustomEventCategory::FeatureUsage
        );
        assert!(super::CustomEventCategory::from_str("invalid").is_err());
    }

    #[test]
    fn test_custom_event_category_string_conversion() {
        let category = super::CustomEventCategory::FeatureUsage;
        let as_string: String = category.into();
        assert_eq!(as_string, "feature_usage");
        
        let back_to_enum = super::CustomEventCategory::try_from(as_string).unwrap();
        assert_eq!(back_to_enum, super::CustomEventCategory::FeatureUsage);
    }
} 