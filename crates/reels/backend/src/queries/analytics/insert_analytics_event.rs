//! Analytics event insertion queries for comprehensive tracking.
//!
//! Provides efficient insertion of analytics events with batch processing support.
//! Handles both individual event insertion and bulk operations for performance.
//! Supports middleware and custom event sources with flexible data structures.
//! Maintains data integrity and provides error handling for tracking operations.

use sqlx::Row;

pub async fn insert_analytics_event(
    pool: &sqlx::PgPool,
    event: crate::db::analytics_events::NewAnalyticsEvent,
) -> Result<uuid::Uuid, sqlx::Error> {
        let result = sqlx::query(
        r#"
        INSERT INTO analytics_events (
            event_name, user_id, request_details, custom_details, session_id
        ) VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(&event.event_name)
    .bind(&event.user_id)
    .bind(&event.request_details)
    .bind(&event.custom_details)
    .bind(&event.session_id)
    .fetch_one(pool)
    .await?;
    
    // Extract the id from the raw query result
    let id: uuid::Uuid = result.get("id");
    Ok(id)
}

/// Insert analytics event and return the complete event (prevents race conditions)
/// This is preferred over insert + fetch for atomicity
pub async fn insert_analytics_event_returning(
    pool: &sqlx::PgPool,
    event: crate::db::analytics_events::NewAnalyticsEvent,
) -> Result<crate::db::analytics_events::AnalyticsEvent, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO analytics_events (
            event_name, user_id, request_details, custom_details, session_id
        ) VALUES ($1, $2, $3, $4, $5)
        RETURNING id, event_name, user_id, timestamp, request_details, custom_details, session_id
        "#,
    )
    .bind(&event.event_name)
    .bind(&event.user_id)
    .bind(&event.request_details)
    .bind(&event.custom_details)
    .bind(&event.session_id)
    .fetch_one(pool)
    .await?;
    
    Ok(crate::db::analytics_events::AnalyticsEvent {
        id: result.get("id"),
        event_name: result.get("event_name"),
        user_id: result.get("user_id"),
        timestamp: result.get("timestamp"),
        request_details: result.get("request_details"),
        custom_details: result.get("custom_details"),
        session_id: result.get("session_id"),
    })
}

pub async fn insert_analytics_events_batch(
    pool: &sqlx::PgPool,
    events: Vec<crate::db::analytics_events::NewAnalyticsEvent>,
) -> Result<Vec<uuid::Uuid>, sqlx::Error> {
    if events.is_empty() {
        return Ok(Vec::new());
    }
    
    let mut transaction = pool.begin().await?;
    let mut inserted_ids = Vec::with_capacity(events.len());
    
    for event in events {
        let result = sqlx::query(
            r#"
            INSERT INTO analytics_events (
                event_name, user_id, request_details, custom_details, session_id
            ) VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
        .bind(&event.event_name)
        .bind(&event.user_id)
        .bind(&event.request_details)
        .bind(&event.custom_details)
        .bind(&event.session_id)
        .fetch_one(&mut *transaction)
        .await?;
        
        let id: uuid::Uuid = result.get("id");
        inserted_ids.push(id);
    }
    
    transaction.commit().await?;
    Ok(inserted_ids)
}

pub async fn get_analytics_event_by_id(
    pool: &sqlx::PgPool,
    event_id: uuid::Uuid,
) -> Result<Option<crate::db::analytics_events::AnalyticsEvent>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT 
            id, event_name, user_id, timestamp, request_details, custom_details, session_id
        FROM analytics_events 
        WHERE id = $1
        "#,
    )
    .bind(event_id)
    .fetch_optional(pool)
    .await?;
    
    match row {
        Some(r) => {
            Ok(Some(crate::db::analytics_events::AnalyticsEvent {
                id: r.get("id"),
                event_name: r.get("event_name"),
                user_id: r.get("user_id"),
                timestamp: r.get("timestamp"),
                request_details: r.get("request_details"),
                custom_details: r.get("custom_details"),
                session_id: r.get("session_id"),
            }))
        }
        None => Ok(None)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_analytics_event_creation() {
        let event = crate::db::analytics_events::NewAnalyticsEvent {
            event_name: String::from("vocal_tour_generated"),
            user_id: Some(uuid::Uuid::new_v4()),
            request_details: serde_json::json!({
                "ip_address": "127.0.0.1",
                "user_agent": "test-agent",
                "browser_info": {"family": "Chrome", "version": "91.0"}
            }),
            custom_details: serde_json::json!({
                "gennodes_server_response": {"status": "success", "processing_time": 5000}
            }),
            session_id: Some(String::from("session_123")),
        };
        
        assert_eq!(event.event_name, "vocal_tour_generated");
        assert_eq!(event.request_details["ip_address"], "127.0.0.1");
    }

    #[test]
    fn test_batch_events_preparation() {
        let events = vec![
            crate::db::analytics_events::NewAnalyticsEvent {
                event_name: String::from("user_login"),
                user_id: Some(uuid::Uuid::new_v4()),
                request_details: serde_json::json!({"ip_address": "127.0.0.1"}),
                custom_details: serde_json::json!({"login_method": "email"}),
                session_id: None,
            },
            crate::db::analytics_events::NewAnalyticsEvent {
                event_name: String::from("listing_created"),
                user_id: Some(uuid::Uuid::new_v4()),
                request_details: serde_json::json!({"ip_address": "192.168.1.1"}),
                custom_details: serde_json::json!({"property_type": "house"}),
                session_id: None,
            }
        ];
        
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_name, "user_login");
        assert_eq!(events[1].event_name, "listing_created");
    }
} 