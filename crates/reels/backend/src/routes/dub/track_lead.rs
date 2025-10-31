//! Handler for tracking lead events via Dub API
//!
//! This endpoint allows the frontend to report lead events for attribution tracking.

use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::instrument;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::services::dub::{DubLeadEvent, DubServiceTrait};

/// Request payload for tracking a lead event
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TrackLeadRequest {
    /// Customer identifier (user UUID)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub customer_id: Uuid,
    /// Customer email address
    #[schema(example = "user@example.com")]
    pub customer_email: String,
    /// Event name (e.g., "Sign Up", "Account Created")
    #[schema(example = "Sign Up")]
    pub event_name: String,
    /// Click ID for attribution (dub_id from URL parameter or cookie)
    #[schema(example = "clk_abc123def456")]
    pub click_id: Option<String>,
    /// Additional metadata for the event
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Response for lead tracking endpoint
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TrackLeadResponse {
    /// Whether the event was successfully tracked
    pub success: bool,
    /// Event ID from Dub (if available)
    pub event_id: Option<String>,
    /// Message from Dub API or error details
    pub message: Option<String>,
}

/// Track a lead event for attribution
#[utoipa::path(
    post,
    path = "/api/dub/track/lead",
    tag = "Dub Attribution",
    request_body = TrackLeadRequest,
    responses(
        (status = 200, description = "Lead event tracked successfully", body = TrackLeadResponse),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
    )
)]
#[post("/lead")]
#[instrument(skip(dub_service, req), fields(customer_id = %req.customer_id, event_name = %req.event_name))]
pub async fn track_lead(
    dub_service: web::Data<dyn DubServiceTrait>,
    req: web::Json<TrackLeadRequest>,
) -> impl Responder {
    // Convert request to DubLeadEvent
    let mut metadata = req.metadata.clone();
    metadata.insert("source".to_string(), serde_json::Value::String("frontend".to_string()));

    let lead_event = DubLeadEvent {
        customer_id: req.customer_id.to_string(),
        customer_email: req.customer_email.clone(),
        event_name: req.event_name.clone(),
        click_id: req.click_id.clone(),
        metadata,
    };

    // Track the event
    match dub_service.track_lead_event(lead_event).await {
        Ok(response) => {
            log::info!(
                "Successfully processed lead tracking request for customer: {} (success: {})",
                req.customer_id,
                response.success
            );

            HttpResponse::Ok().json(TrackLeadResponse {
                success: response.success,
                event_id: response.event_id,
                message: response.message,
            })
        }
        Err(e) => {
            log::error!(
                "Failed to track lead event for customer: {} - {}",
                req.customer_id,
                e
            );

            HttpResponse::InternalServerError().json(TrackLeadResponse {
                success: false,
                event_id: None,
                message: Some(format!("Failed to track lead event: {}", e)),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use async_trait::async_trait;
    use std::sync::Arc;

    // Mock implementation for testing
    struct MockDubService {
        should_succeed: bool,
    }

    #[async_trait]
    impl DubServiceTrait for MockDubService {
        async fn track_lead_event(
            &self,
            _event: DubLeadEvent,
        ) -> anyhow::Result<crate::services::dub::DubEventResponse> {
            if self.should_succeed {
                Ok(crate::services::dub::DubEventResponse {
                    success: true,
                    event_id: Some("test_event_123".to_string()),
                    message: Some("Success".to_string()),
                })
            } else {
                Err(anyhow::anyhow!("Mock failure"))
            }
        }

        async fn track_sale_event(
            &self,
            _event: crate::services::dub::DubSaleEvent,
        ) -> anyhow::Result<crate::services::dub::DubEventResponse> {
            unimplemented!()
        }

        fn is_enabled(&self) -> bool {
            true
        }

        fn get_workspace_id(&self) -> Option<String> {
            Some("test_workspace".to_string())
        }

        async fn generate_referral_token(
            &self,
            _tenant_id: String,
            _partner_name: String,
            _partner_email: String,
            _partner_image: Option<String>,
            _group_id: Option<String>,
        ) -> anyhow::Result<String> {
            if self.should_succeed {
                Ok("test_referral_token_123".to_string())
            } else {
                Err(anyhow::anyhow!("Mock referral token generation failure"))
            }
        }
    }

    #[actix_web::test]
    async fn test_track_lead_success() {
        let mock_service = Arc::new(MockDubService { should_succeed: true });
        let app = test::init_service(
            App::new()
                .app_data(web::Data::from(mock_service as Arc<dyn DubServiceTrait>))
                .service(web::scope("/api/dub/track").service(track_lead)),
        )
        .await;

        let request_payload = TrackLeadRequest {
            customer_id: Uuid::new_v4(),
            customer_email: "test@example.com".to_string(),
            event_name: "Sign Up".to_string(),
            click_id: Some("test-click-id".to_string()),
            metadata: HashMap::new(),
        };

        let req = test::TestRequest::post()
            .uri("/api/dub/track/lead")
            .set_json(&request_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let response_body: TrackLeadResponse = test::read_body_json(resp).await;
        assert!(response_body.success);
        assert_eq!(response_body.event_id, Some("test_event_123".to_string()));
    }

    #[actix_web::test]
    async fn test_track_lead_failure() {
        let mock_service = Arc::new(MockDubService { should_succeed: false });
        let app = test::init_service(
            App::new()
                .app_data(web::Data::from(mock_service as Arc<dyn DubServiceTrait>))
                .service(web::scope("/api/dub/track").service(track_lead)),
        )
        .await;

        let request_payload = TrackLeadRequest {
            customer_id: Uuid::new_v4(),
            customer_email: "test@example.com".to_string(),
            event_name: "Sign Up".to_string(),
            click_id: Some("test-click-id".to_string()),
            metadata: HashMap::new(),
        };

        let req = test::TestRequest::post()
            .uri("/api/dub/track/lead")
            .set_json(&request_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_server_error());

        let response_body: TrackLeadResponse = test::read_body_json(resp).await;
        assert!(!response_body.success);
        assert!(response_body.message.is_some());
    }
}
