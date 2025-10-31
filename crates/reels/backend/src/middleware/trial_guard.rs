use crate::middleware::auth::AuthenticatedUser;
use crate::services::trial_service::has_access;
use actix_web::{
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures::future::{ok, LocalBoxFuture, Ready};
use sqlx::PgPool;
use std::{
    sync::Arc,
    task::{Context, Poll},
};
use tracing::instrument;

/// Middleware to check if user has access (trial active or subscription active)
#[derive(Clone)]
pub struct TrialGuard;

impl<S> Transform<S, ServiceRequest> for TrialGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = TrialGuardService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(TrialGuardService {
            service: Arc::new(service),
        })
    }
}

pub struct TrialGuardService<S> {
    service: Arc<S>,
}

impl<S> Service<ServiceRequest> for TrialGuardService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    #[instrument(skip(self, req))]
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let pool_data = req.app_data::<actix_web::web::Data<PgPool>>().cloned();
        let srv = self.service.clone();

        Box::pin(async move {
            // Get the authenticated user from the request extensions
            let user_id = req.extensions().get::<AuthenticatedUser>().map(|auth_user| {
                match auth_user {
                    AuthenticatedUser::Jwt(claims) => claims.user_id,
                    AuthenticatedUser::ApiKey(user_id) => *user_id,
                }
            });

            let user_id = if let Some(user_id) = user_id {
                user_id
            } else {
                // No authenticated user, let the service handle it
                return srv.call(req).await;
            };

            // Check if user has access (trial active or subscription active)
            if let Some(pool) = pool_data {
                match has_access(&pool, user_id).await {
                    Ok(has_access) => {
                        if has_access {
                            srv.call(req).await
                        } else {
                            // Trial expired and no active subscription
                            let response = HttpResponse::Forbidden()
                                .json(serde_json::json!({
                                    "error": "Trial expired",
                                    "message": "Your trial has expired. Please upgrade to continue using the service.",
                                    "code": "TRIAL_EXPIRED"
                                }));
                            Ok(req.into_response(response.map_into_boxed_body()))
                        }
                    }
                    Err(_) => {
                        // Database error, let the service handle it
                        srv.call(req).await
                    }
                }
            } else {
                // No database pool, let the service handle it
                srv.call(req).await
            }
        })
    }
}

/// Middleware to allow access to billing settings even if trial expired
#[derive(Clone)]
pub struct BillingAccessGuard;

impl<S> Transform<S, ServiceRequest> for BillingAccessGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = BillingAccessGuardService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(BillingAccessGuardService {
            service: Arc::new(service),
        })
    }
}

pub struct BillingAccessGuardService<S> {
    service: Arc<S>,
}

impl<S> Service<ServiceRequest> for BillingAccessGuardService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    #[instrument(skip(self, req))]
    fn call(&self, req: ServiceRequest) -> Self::Future {
        // For billing routes, we allow access even if trial expired
        // This allows users to access billing settings to upgrade
        let srv = self.service.clone();
        Box::pin(async move {
            srv.call(req).await
        })
    }
} 