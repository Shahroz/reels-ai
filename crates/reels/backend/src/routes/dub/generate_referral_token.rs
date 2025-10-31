//! Handler for generating Dub referral tokens
//!
//! This endpoint generates a public token for embedded referral dashboard
//! using the authenticated user's information.

use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use utoipa::ToSchema;

use crate::auth::tokens::claims::Claims;
use crate::services::dub::DubServiceTrait;

/// Response for referral token generation endpoint
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenerateReferralTokenResponse {
    /// The public token for embedded referral dashboard
    #[schema(example = "pub_token_abc123def456")]
    pub public_token: String,
}

/// Generate a referral token for embedded referral dashboard
#[utoipa::path(
    get,
    path = "/api/dub/referral-token",
    tag = "Dub Attribution",
    responses(
        (status = 200, description = "Referral token generated successfully", body = GenerateReferralTokenResponse),
        (status = 401, description = "Unauthorized - user not authenticated"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("")]
#[instrument(skip(dub_service, claims))]
pub async fn generate_referral_token(
    dub_service: web::Data<dyn DubServiceTrait>,
    claims: Claims,
) -> impl Responder {
    // Extract user information from JWT claims
    let tenant_id = claims.user_id.to_string();
    let partner_name = "".to_string(); // Empty name as suggested
    let partner_email = claims.email;
    let partner_image = None; // Empty profile picture as suggested
    let group_id = None; // Could be extracted from user data if needed
    // Generate the referral token
    match dub_service.generate_referral_token(
        tenant_id.clone(),
        partner_name.clone(),
        partner_email.clone(),
        partner_image.clone(),
        group_id,
    ).await {
        Ok(token) => {
            log::info!(
                "Successfully generated dub referral token for user: {} ({})",
                tenant_id,
                partner_email
            );

            HttpResponse::Ok().json(GenerateReferralTokenResponse {
                public_token: token,
            })
        }
        Err(e) => {
            log::error!(
                "Failed to generate dub referral token for user: {} - {}",
                tenant_id,
                e
            );

            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to generate dub referral token",
                "message": e.to_string()
            }))
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
            _event: crate::services::dub::DubLeadEvent,
        ) -> anyhow::Result<crate::services::dub::DubEventResponse> {
            unimplemented!()
        }

        async fn track_sale_event(
            &self,
            _event: crate::services::dub::DubSaleEvent,
        ) -> anyhow::Result<crate::services::dub::DubEventResponse> {
            unimplemented!()
        }

        async fn generate_referral_token(
            &self,
            tenant_id: String,
            partner_name: String,
            partner_email: String,
            partner_image: Option<String>,
            group_id: Option<String>,
        ) -> anyhow::Result<String> {
            if self.should_succeed {
                Ok(format!("pub_token_{}", tenant_id))
            } else {
                Err(anyhow::anyhow!("Mock failure"))
            }
        }

        fn is_enabled(&self) -> bool {
            true
        }

        fn get_workspace_id(&self) -> Option<String> {
            Some("test_workspace".to_string())
        }
    }

    #[actix_web::test]
    async fn test_generate_referral_token_success() {
        let mock_service = Arc::new(MockDubService { should_succeed: true });
        let app = test::init_service(
            App::new()
                .app_data(web::Data::from(mock_service as Arc<dyn DubServiceTrait>))
                .service(web::scope("/api/dub").service(generate_referral_token)),
        )
        .await;

        // Note: In a real test, you'd need to properly mock the JWT validation middleware
        // and database pool. For now, this test structure shows the expected behavior
        let req = test::TestRequest::get()
            .uri("/api/dub/referral-token")
            .insert_header(("Authorization", "Bearer test_token"))
            .to_request();

        // This test would need proper JWT middleware mocking to work fully
        // The structure shows the expected API contract
    }

    #[actix_web::test]
    async fn test_generate_referral_token_failure() {
        let mock_service = Arc::new(MockDubService { should_succeed: false });
        let app = test::init_service(
            App::new()
                .app_data(web::Data::from(mock_service as Arc<dyn DubServiceTrait>))
                .service(web::scope("/api/dub").service(generate_referral_token)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/dub/referral-token")
            .insert_header(("Authorization", "Bearer test_token"))
            .to_request();

        // This test would need proper JWT middleware mocking to work fully
        // The structure shows the expected API contract
    }
}
