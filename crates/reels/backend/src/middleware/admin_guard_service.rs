//! Service implementation for AdminGuard middleware.
//!
//! This file contains the Service trait implementation for the AdminGuard middleware,
//! handling the actual request processing logic. It checks request extensions for
//! authenticated user information and enforces admin authorization. Must be used in
//! conjunction with admin_guard.rs which provides the Transform implementation.
//!
//! Revision History:
//! - 2025-10-10: Split from admin_guard.rs to comply with one-item-per-file pattern.

pub struct AdminGuardService<S> {
    pub(crate) service: std::sync::Arc<S>,
}

// Required to access request extensions
use actix_web::HttpMessage;

impl<S> actix_web::dev::Service<actix_web::dev::ServiceRequest> for AdminGuardService<S>
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
            // Get the authenticated user from the request extensions (set by JwtMiddleware)
            let is_admin = req
                .extensions()
                .get::<crate::middleware::auth::AuthenticatedUser>()
                .map(|auth_user| match auth_user {
                    crate::middleware::auth::AuthenticatedUser::Jwt(claims) => claims.is_admin,
                    crate::middleware::auth::AuthenticatedUser::ApiKey(_) => false, // API keys don't have admin privileges
                });

            match is_admin {
                Some(true) => {
                    // User is an admin, allow the request to proceed
                    srv.call(req).await
                }
                Some(false) => {
                    // User is authenticated but not an admin - return 403 Forbidden
                    log::warn!("Non-admin user attempted to access admin endpoint");
                    let (req, _) = req.into_parts();
                    let response = actix_web::HttpResponse::Forbidden()
                        .json(serde_json::json!({
                            "error": "User is not authorized to perform this action."
                        }))
                        .map_into_boxed_body();
                    Ok(actix_web::dev::ServiceResponse::new(req, response))
                }
                None => {
                    // No authenticated user found - this shouldn't happen if JwtMiddleware is applied first
                    // Return 401 Unauthorized as authentication is missing
                    log::error!("AdminGuard called without authenticated user - ensure JwtMiddleware is applied first");
                    let (req, _) = req.into_parts();
                    let response = actix_web::HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "Authentication required."
                        }))
                        .map_into_boxed_body();
                    Ok(actix_web::dev::ServiceResponse::new(req, response))
                }
            }
        })
    }
}

