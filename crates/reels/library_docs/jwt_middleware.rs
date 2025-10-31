// use std::{
//     env,
//     task::{Context, Poll},
// };
//
// use actix_web::{
//     body::BoxBody,
//     dev::{Service, ServiceRequest, ServiceResponse, Transform},
//     Error,
//     FromRequest, http::header, HttpMessage, HttpRequest, HttpResponse,
// };
// use actix_web::dev::Payload;
// use anyhow::anyhow;
// use futures::future::{LocalBoxFuture, ok, Ready, ready};
// use gcp_bigquery_client::model::training_options::HolidayRegion::Ma;
// use jsonwebtoken::{Algorithm, decode, DecodingKey, encode, EncodingKey, Header, Validation};
// use serde::{Deserialize, Serialize};
//
// // Define your Claims struct
// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct Claims {
//     pub tenant_id: String,
//     exp: usize,
//     role: Option<String>,
// }
//
// impl Claims {
//     /// Create a new `Claims` instance for testing based on the provided `tenant_id`.
//     /// This can be helpful in tests where you want to generate a JWT with known claims.
//     pub fn new_with_tenant_id(tenant_id: &str) -> Self {
//         // For testing, let's set the expiration to one hour from now.
//         // You can adjust this as needed.
//         let one_hour_from_now = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize;
//
//         Claims {
//             tenant_id: tenant_id.to_string(),
//             exp: one_hour_from_now,
//             role: Some("test_role".into()), // You can use any fixed role for testing
//         }
//     }
//
//     pub fn to_token(&self) -> anyhow::Result<String> {
//         let secret_key = env::var("JWT_SECRET").map_err(|_| {
//             jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidKeyFormat)
//         })?;
//
//         let token = encode(
//             &Header::default(),
//             self,
//             &EncodingKey::from_secret(secret_key.as_ref())
//         ).map_err(|e| anyhow!("{}", e.to_string()))?;
//         Ok(format!("Bearer {}", token))
//     }
// }
//
// #[derive(Debug, Clone)]
// pub struct MaybeClaims(pub Option<Claims>);
//
// // Define the middleware struct
// pub struct JwtMiddleware;
//
// impl<S> Transform<S, ServiceRequest> for JwtMiddleware
//     where
//     // Define the expected service type
//         S: Service<ServiceRequest, Response=ServiceResponse<BoxBody>, Error=Error> + 'static,
//     // Ensure the future returned by the service is 'static
//         S::Future: 'static,
// {
//     type Response = ServiceResponse<BoxBody>;
//     type Error = Error;
//     type Transform = JwtMiddlewareService<S>;
//     type InitError = ();
//     // The middleware initialization is immediate, so we use Ready
//     type Future = Ready<Result<Self::Transform, Self::InitError>>;
//
//     // Create and return a new instance of the middleware service
//     fn new_transform(&self, service: S) -> Self::Future {
//         ok(JwtMiddlewareService { service })
//     }
// }
//
// // Define the middleware service struct
// pub struct JwtMiddlewareService<S> {
//     service: S,
// }
//
// impl<S> Service<ServiceRequest> for JwtMiddlewareService<S>
//     where
//         S: Service<ServiceRequest, Response=ServiceResponse<BoxBody>, Error=Error> + 'static,
//         S::Future: 'static,
// {
//     type Response = ServiceResponse<BoxBody>;
//     type Error = Error;
//     type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;
//
//     fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         self.service.poll_ready(ctx)
//     }
//
//     fn call(&self, mut req: ServiceRequest) -> Self::Future {
//         let path = req.path().to_string();
//         log::info!("JWT verification for path: {}", path);
//
//         let secret_key = match env::var("JWT_SECRET") {
//             Ok(key) => key,
//             Err(_) => {
//                 log::error!("JWT_SECRET not set in environment variables");
//                 let response = HttpResponse::InternalServerError().finish();
//                 return Box::pin(async move { Ok(req.into_response(response)) });
//             }
//         };
//
//         // Extract and verify the Authorization header
//         let auth_header = match req
//             .headers()
//             .get(header::AUTHORIZATION)
//             .and_then(|h| h.to_str().ok())
//         {
//             Some(header) => header,
//             None => {
//                 log::info!("No Authorization header present");
//                 let response = HttpResponse::Unauthorized().finish();
//                 return Box::pin(async move { Ok(req.into_response(response)) });
//             }
//         };
//
//         if !auth_header.starts_with("Bearer ") {
//             log::warn!("Authorization header does not start with 'Bearer '");
//             let response = HttpResponse::Unauthorized().finish();
//             return Box::pin(async move { Ok(req.into_response(response)) });
//         }
//
//         let token = &auth_header[7..];
//         let validation = Validation::new(Algorithm::HS256);
//
//         match decode::<Claims>(
//             token,
//             &DecodingKey::from_secret(secret_key.as_bytes()),
//             &validation,
//         ) {
//             Ok(data) => {
//                 log::info!("JWT verified successfully: {:?}", data.claims);
//                 // Insert MaybeClaims(Some(claims)) into request extensions
//                 req.extensions_mut().insert(MaybeClaims(Some(data.claims)));
//             }
//             Err(err) => {
//                 log::error!("JWT verification failed: {:?}", err);
//                 let response = HttpResponse::Unauthorized().finish();
//                 return Box::pin(async move { Ok(req.into_response(response)) });
//             }
//         }
//
//         // Forward the request to the next service
//         let fut = self.service.call(req);
//         Box::pin(async move { fut.await })
//     }
// }
//
// impl FromRequest for MaybeClaims {
//     type Error = actix_web::Error;
//     type Future = Ready<Result<Self, Self::Error>>;
//
//     fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
//         if let Some(maybe_claims) = req.extensions().get::<MaybeClaims>() {
//             ready(Ok(maybe_claims.clone()))
//         } else {
//             ready(Ok(MaybeClaims(None)))
//         }
//     }
// }
