//! Middleware to enforce admin authorization for protected routes.
//!
//! This middleware checks if the authenticated user has admin privileges by examining
//! the is_admin flag in their JWT claims. It should be applied to routes that require
//! administrative access. Returns 403 Forbidden if the user is authenticated but not
//! an admin, allowing the request to proceed only if the user has admin privileges.
//!
//! Revision History:
//! - 2025-10-10: Split into admin_guard.rs and admin_guard_service.rs for one-item-per-file.

/// Middleware to check if the authenticated user has admin privileges.
/// Must be applied after JwtMiddleware to access the authenticated user.
#[derive(Clone)]
pub struct AdminGuard;

impl<S> actix_web::dev::Transform<S, actix_web::dev::ServiceRequest> for AdminGuard
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
    type Transform = crate::middleware::admin_guard_service::AdminGuardService<S>;
    type InitError = ();
    type Future = futures::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        futures::future::ok(crate::middleware::admin_guard_service::AdminGuardService {
            service: std::sync::Arc::new(service),
        })
    }
}
