//! Service implementation for ImageboardWebhookGuard middleware.
//!
//! This file contains the Service trait implementation for the ImageboardWebhookGuard middleware,
//! handling the actual request processing logic. It extracts collection_id from path and validates
//! organization and collection existence.

use actix_web::HttpMessage;
use actix_web::web::Data;
use sqlx::PgPool;

pub struct ImageboardWebhookGuardService<S> {
    pub(crate) service: std::sync::Arc<S>,
}

impl<S> actix_web::dev::Service<actix_web::dev::ServiceRequest> for ImageboardWebhookGuardService<S>
where
    S: actix_web::dev::Service<
            actix_web::dev::ServiceRequest,
            Response = actix_web::dev::ServiceResponse<actix_web::body::BoxBody>,
            Error = actix_web::Error,
        > + 'static,
    S::Future: 'static,
{
    type Response = actix_web::dev::ServiceResponse<actix_web::body::BoxBody>;
    type Error = actix_web::Error;
    type Future = futures::future::LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    #[tracing::instrument(skip(self, req))]
    fn call(&self, req: actix_web::dev::ServiceRequest) -> Self::Future {
        let srv = self.service.clone();

        Box::pin(async move {
            // Get database pool from app data
            let pool = req.app_data::<Data<PgPool>>();
            let pool = match pool {
                Some(p) => p.as_ref(),
                None => {
                    log::error!("ImageboardWebhookGuard: Database pool not found in app data");
                    let (req, _) = req.into_parts();
                    let response = actix_web::HttpResponse::InternalServerError()
                        .json(serde_json::json!({
                            "error": "Internal server error",
                            "code": "INTERNAL_SERVER_ERROR",
                            "details": "Database pool not configured"
                        }))
                        .map_into_boxed_body();
                    return Ok(actix_web::dev::ServiceResponse::new(req, response));
                }
            };

            // Extract collection_id from path parameter
            // Path format: /imageboard/webhook/{collection_id}/...
            let path = req.path();
            let collection_id_str = if let Some(start_idx) = path.find("/webhook/") {
                let after_webhook = &path[start_idx + 9..]; // "/webhook/".len() = 9
                if let Some(end_idx) = after_webhook.find('/') {
                    &after_webhook[..end_idx]
                } else {
                    after_webhook
                }
            } else {
                log::error!("ImageboardWebhookGuard: Collection ID not found in path: {}", path);
                let (req, _) = req.into_parts();
                let response = actix_web::HttpResponse::BadRequest()
                    .json(serde_json::json!({
                        "error": "Missing collection ID in path",
                        "code": "MISSING_COLLECTION_ID",
                        "details": None::<String>
                    }))
                    .map_into_boxed_body();
                return Ok(actix_web::dev::ServiceResponse::new(req, response));
            };

            // Parse collection_id as UUID
            let board_id = match uuid::Uuid::parse_str(collection_id_str) {
                Ok(id) => id,
                Err(e) => {
                    log::warn!("ImageboardWebhookGuard: Invalid collection ID format: {}", e);
                    let (req, _) = req.into_parts();
                    let response = actix_web::HttpResponse::BadRequest()
                        .json(serde_json::json!({
                            "error": format!("Invalid collection ID format: {}", e),
                            "code": "INVALID_COLLECTION_ID",
                            "details": None::<String>
                        }))
                        .map_into_boxed_body();
                    return Ok(actix_web::dev::ServiceResponse::new(req, response));
                }
            };

            // Fetch collection (board) to get organization_id
            let collection = match crate::queries::collections::get_collection_by_id::get_collection_by_id(pool, board_id).await {
                Ok(Some(collection)) => collection,
                Ok(None) => {
                    log::warn!("ImageboardWebhookGuard: Collection (board) not found: {}", board_id);
                    let (req, _) = req.into_parts();
                    let response = actix_web::HttpResponse::NotFound()
                        .json(serde_json::json!({
                            "error": "Board not found",
                            "code": "BOARD_NOT_FOUND",
                            "details": None::<String>
                        }))
                        .map_into_boxed_body();
                    return Ok(actix_web::dev::ServiceResponse::new(req, response));
                }
                Err(e) => {
                    log::error!("ImageboardWebhookGuard: Error checking collection: {:?}", e);
                    let (req, _) = req.into_parts();
                    let response = actix_web::HttpResponse::InternalServerError()
                        .json(serde_json::json!({
                            "error": "Internal server error",
                            "code": "INTERNAL_SERVER_ERROR",
                            "details": Some(format!("Failed to check board: {}", e))
                        }))
                        .map_into_boxed_body();
                    return Ok(actix_web::dev::ServiceResponse::new(req, response));
                }
            };

            // Extract organization_id from collection
            let organization_id = match collection.organization_id {
                Some(org_id) => org_id,
                None => {
                    log::warn!("ImageboardWebhookGuard: Collection does not have an organization linked: {}", board_id);
                    let (req, _) = req.into_parts();
                    let response = actix_web::HttpResponse::BadRequest()
                        .json(serde_json::json!({
                            "error": "Board does not have an organization linked",
                            "code": "BOARD_NO_ORGANIZATION",
                            "details": Some(board_id.to_string())
                        }))
                        .map_into_boxed_body();
                    return Ok(actix_web::dev::ServiceResponse::new(req, response));
                }
            };

            // Check organization existence
            match crate::queries::organizations::organization_exists::organization_exists(pool, organization_id).await {
                Ok(true) => {},
                Ok(false) => {
                    log::warn!("ImageboardWebhookGuard: Organization not found: {}", organization_id);
                    let (req, _) = req.into_parts();
                    let response = actix_web::HttpResponse::NotFound()
                        .json(serde_json::json!({
                            "error": "Organization not found",
                            "code": "ORGANIZATION_NOT_FOUND",
                            "details": None::<String>
                        }))
                        .map_into_boxed_body();
                    return Ok(actix_web::dev::ServiceResponse::new(req, response));
                }
                Err(e) => {
                    log::error!("ImageboardWebhookGuard: Error checking organization: {:?}", e);
                    let (req, _) = req.into_parts();
                    let response = actix_web::HttpResponse::InternalServerError()
                        .json(serde_json::json!({
                            "error": "Internal server error",
                            "code": "INTERNAL_SERVER_ERROR",
                            "details": Some(format!("Failed to check organization: {}", e))
                        }))
                        .map_into_boxed_body();
                    return Ok(actix_web::dev::ServiceResponse::new(req, response));
                }
            }

            // Store collection_id (board_id) in request extensions for handlers to use
            req.extensions_mut().insert(board_id);

            // All checks passed, proceed with the request
            srv.call(req).await
        })
    }
}

