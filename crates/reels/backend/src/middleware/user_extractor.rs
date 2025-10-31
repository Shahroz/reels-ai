// backend/src/middleware/user_extractor.rs
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpResponse};
use futures_util::future::{ok, Ready, LocalBoxFuture};
use crate::auth::tokens::verify_jwt;
use uuid::Uuid;
use tracing::instrument;

pub struct UserExtractor;

impl<S, B> Transform<S, ServiceRequest> for UserExtractor
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = UserExtractorMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(UserExtractorMiddleware { service })
    }
}

pub struct UserExtractorMiddleware<S> {
    pub service: S,
}

impl<S, B> Service<ServiceRequest> for UserExtractorMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    #[instrument(skip(self, req))]
    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        if let Some(auth_header) = req.headers().get("Authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];
                    if let Ok(claims) = verify_jwt(token) {
                        req.extensions_mut().insert(claims.sub);
                    }
                }
            }
        }
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
