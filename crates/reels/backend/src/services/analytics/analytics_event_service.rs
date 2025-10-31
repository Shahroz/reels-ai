//! Analytics event service for event tracking operations.
//!
//! Provides high-level service methods for analytics event management.
//! Handles event creation, validation, and batch processing.
//! Coordinates between middleware and database for event persistence.
//! Ensures data consistency and provides error handling for event operations.

pub struct AnalyticsEventService {
    db_pool: sqlx::PgPool,
}

impl AnalyticsEventService {
    pub fn new(db_pool: sqlx::PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn create_event(
        &self,
        event: crate::db::analytics_events::NewAnalyticsEvent,
    ) -> Result<uuid::Uuid, EventServiceError> {
        // Validate event data
        self.validate_event(&event)?;

        let event_id = crate::queries::analytics::insert_analytics_event(&self.db_pool, event)
            .await
            .map_err(EventServiceError::DatabaseError)?;

        Ok(event_id)
    }

    pub async fn create_events_batch(
        &self,
        events: Vec<crate::db::analytics_events::NewAnalyticsEvent>,
    ) -> Result<Vec<uuid::Uuid>, EventServiceError> {
        // Validate batch size
        if events.is_empty() {
            return Err(EventServiceError::EmptyBatch);
        }

        if events.len() > 1000 {
            return Err(EventServiceError::BatchTooLarge);
        }

        // Validate all events in the batch
        for event in &events {
            self.validate_event(event)?;
        }

        let event_ids = crate::queries::analytics::insert_analytics_events_batch(&self.db_pool, events)
            .await
            .map_err(EventServiceError::DatabaseError)?;

        Ok(event_ids)
    }

    pub async fn get_event_by_id(
        &self,
        event_id: uuid::Uuid,
    ) -> Result<Option<crate::db::analytics_events::AnalyticsEvent>, EventServiceError> {
        let event = crate::queries::analytics::get_analytics_event_by_id(&self.db_pool, event_id)
            .await
            .map_err(EventServiceError::DatabaseError)?;

        Ok(event)
    }

    // REMOVED: create_middleware_event - no longer needed with custom-only events

    pub async fn create_custom_event(
        &self,
        event_name: String,
        user_id: uuid::Uuid,
        request_details: serde_json::Value,
        session_id: Option<String>,
    ) -> Result<uuid::Uuid, EventServiceError> {

        let event = crate::db::analytics_events::NewAnalyticsEvent {
            event_name,
            user_id: Some(user_id),
            request_details,
            custom_details: serde_json::Value::Null, // Default NULL, should be provided via dedicated method
            session_id,
        };

        self.create_event(event).await
    }

    /// Create a custom event with both request context and custom details
    /// Request details maintain compatibility with middleware events for analytics segmentation
    /// Custom details contain domain-specific data (e.g., gennodes_server_response)
    pub async fn create_custom_event_with_details(
        &self,
        event_name: String,
        user_id: uuid::Uuid,
        request_details: serde_json::Value,   // Request context (ip_address, user_agent, browser_info, etc.)
        custom_details: serde_json::Value,    // Custom event specific data
        session_id: Option<String>,
    ) -> Result<uuid::Uuid, EventServiceError> {
        let event = crate::db::analytics_events::NewAnalyticsEvent {
            event_name,
            user_id: Some(user_id),
            request_details,
            custom_details,
            session_id,
        };

        self.create_event(event).await
    }

    /// Create a custom event and return the complete event atomically (prevents race conditions)
    /// This is preferred over create + fetch for endpoints that need to return the event
    pub async fn create_custom_event_with_details_returning(
        &self,
        event_name: String,
        user_id: uuid::Uuid,
        request_details: serde_json::Value,
        custom_details: serde_json::Value,
        session_id: Option<String>,
    ) -> Result<crate::db::analytics_events::AnalyticsEvent, EventServiceError> {
        let event = crate::db::analytics_events::NewAnalyticsEvent {
            event_name,
            user_id: Some(user_id),
            request_details,
            custom_details,
            session_id,
        };

        // Validate event data
        self.validate_event(&event)?;

        let created_event = crate::queries::analytics::insert_analytics_event_returning(&self.db_pool, event)
            .await
            .map_err(EventServiceError::DatabaseError)?;

        Ok(created_event)
    }

    fn validate_event(&self, event: &crate::db::analytics_events::NewAnalyticsEvent) -> Result<(), EventServiceError> {
        // Validate event name
        if event.event_name.trim().is_empty() {
            return Err(EventServiceError::EmptyEventName);
        }

        if event.event_name.len() > 255 {
            return Err(EventServiceError::EventNameTooLong);
        }

        // Validate session ID if present
        if let Some(ref session_id) = event.session_id {
            if session_id.len() > 255 {
                return Err(EventServiceError::SessionIdTooLong);
            }
        }

        // Validate IP address in request_details if present
        if let Some(ip_value) = event.request_details.get("ip_address") {
            if let Some(ip_str) = ip_value.as_str() {
                if ip_str.len() > 45 { // Max length for IPv6
                    return Err(EventServiceError::InvalidIpAddress);
                }
            }
        }

        // Validate user agent in request_details if present
        if let Some(ua_value) = event.request_details.get("user_agent") {
            if let Some(ua_str) = ua_value.as_str() {
                if ua_str.len() > 1000 {
                    return Err(EventServiceError::UserAgentTooLong);
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum EventServiceError {
    EmptyEventName,
    EventNameTooLong,
    SessionIdTooLong,
    InvalidIpAddress,
    UserAgentTooLong,
    EmptyBatch,
    BatchTooLarge,
    DatabaseError(sqlx::Error),
}

impl std::fmt::Display for EventServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventServiceError::EmptyEventName => write!(f, "Event name cannot be empty"),
            EventServiceError::EventNameTooLong => write!(f, "Event name cannot exceed 255 characters"),
            EventServiceError::SessionIdTooLong => write!(f, "Session ID cannot exceed 255 characters"),
            EventServiceError::InvalidIpAddress => write!(f, "Invalid IP address format"),
            EventServiceError::UserAgentTooLong => write!(f, "User agent cannot exceed 1000 characters"),
            EventServiceError::EmptyBatch => write!(f, "Event batch cannot be empty"),
            EventServiceError::BatchTooLarge => write!(f, "Event batch cannot exceed 1000 events"),
            EventServiceError::DatabaseError(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl std::error::Error for EventServiceError {}

#[cfg(test)]
mod tests {
    #[test]
    fn test_event_service_error_display() {
        let error = super::EventServiceError::EmptyEventName;
        assert_eq!(error.to_string(), "Event name cannot be empty");
    }

    #[test]
    fn test_event_name_validation() {
        let empty_name = "";
        let long_name = "a".repeat(256);
        
        assert!(empty_name.trim().is_empty());
        assert!(long_name.len() > 255);
    }

    #[test]
    fn test_batch_size_validation() {
        let empty_batch: Vec<i32> = vec![];
        let large_batch: Vec<i32> = (0..1001).collect();
        
        assert!(empty_batch.is_empty());
        assert!(large_batch.len() > 1000);
    }
} 