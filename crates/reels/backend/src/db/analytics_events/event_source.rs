//! Event source enum for analytics events with database string storage.
//!
//! Defines the source of analytics events: middleware (automatic) or custom (manual).
//! Stored as string in database for flexibility while maintaining type safety in Rust.
//! Provides conversion methods for database integration and validation.
//! Used throughout the analytics system for event categorization.

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum EventSource {
    Middleware,
    Custom,
}

impl EventSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventSource::Middleware => "middleware",
            EventSource::Custom => "custom",
        }
    }
    
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "middleware" => Ok(EventSource::Middleware),
            "custom" => Ok(EventSource::Custom),
            _ => Err(std::format!("Invalid event source: {}", s)),
        }
    }
}

impl std::convert::TryFrom<String> for EventSource {
    type Error = String;
    
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl std::convert::From<EventSource> for String {
    fn from(source: EventSource) -> Self {
        source.as_str().to_string()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_event_source_as_str() {
        assert_eq!(super::EventSource::Middleware.as_str(), "middleware");
        assert_eq!(super::EventSource::Custom.as_str(), "custom");
    }

    #[test]
    fn test_event_source_from_str() {
        assert_eq!(
            super::EventSource::from_str("middleware").unwrap(),
            super::EventSource::Middleware
        );
        assert_eq!(
            super::EventSource::from_str("custom").unwrap(),
            super::EventSource::Custom
        );
        assert!(super::EventSource::from_str("invalid").is_err());
    }

    #[test]
    fn test_event_source_string_conversion() {
        let source = super::EventSource::Middleware;
        let as_string: String = source.into();
        assert_eq!(as_string, "middleware");
        
        let back_to_enum = super::EventSource::try_from(as_string).unwrap();
        assert_eq!(back_to_enum, super::EventSource::Middleware);
    }
} 