//! Middleware to guard Imageboard webhook routes.
//!
//! This middleware:
//! 1. Extracts collection_id from path parameter `{collection_id}`
//! 2. Validates collection (board) existence
//! 3. Validates organization existence (via collection's organization_id)
//!
//! The collection data is stored in request extensions for handlers to use.

/// Middleware to guard Imageboard webhook routes.
/// Must extract collection_id from path parameter and validate organization/collection.
#[derive(Clone)]
pub struct ImageboardWebhookGuard;

impl<S> actix_web::dev::Transform<S, actix_web::dev::ServiceRequest> for ImageboardWebhookGuard
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
    type Transform = crate::middleware::imageboard_webhook_guard_service::ImageboardWebhookGuardService<S>;
    type InitError = ();
    type Future = futures::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        futures::future::ok(crate::middleware::imageboard_webhook_guard_service::ImageboardWebhookGuardService {
            service: std::sync::Arc::new(service),
        })
    }
}

