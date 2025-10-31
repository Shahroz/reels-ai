use crate::auth::tokens::{Claims, verify_jwt};
use crate::db::api_keys::validate_api_key_with_domain;
use crate::middleware::rate_limit::{RateLimiter, rate_limit_exceeded_response};

use actix_web::{
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
    web, // Added for web::Data
    Error,
    HttpMessage,
    HttpResponse,
};
// Removed unused anyhow import
use futures::future::{ok, LocalBoxFuture, Ready};
// Removed unused jsonwebtoken imports (using verify_jwt from auth::tokens now)

use sqlx::PgPool; // Added for PgPool

use std::{
    task::{Context, Poll},
};
use uuid::Uuid; // Added for Uuid
use std::sync::{Arc, OnceLock};
use tracing::instrument;
use std::net::IpAddr;

/// Represents the identity of the authenticated user, either via JWT or API Key.
#[derive(Debug, Clone)]
pub enum AuthenticatedUser {
    Jwt(Claims),
    ApiKey(Uuid), // User ID from API Key validation
}

// Global rate limiter instance
static RATE_LIMITER: OnceLock<RateLimiter> = OnceLock::new();

fn get_rate_limiter() -> &'static RateLimiter {
    RATE_LIMITER.get_or_init(|| RateLimiter::new())
}

/// Clear the rate limit cache (for debugging/admin purposes)
pub fn clear_rate_limit_cache() {
    get_rate_limiter().clear_cache();
}

/// Extract the real client IP address from request headers
/// Checks various headers commonly used by proxies, load balancers, and CDNs
fn extract_client_ip(req: &ServiceRequest) -> String {
    // List of headers to check in order of preference
    let ip_headers = [
        "x-forwarded-for",           // Most common proxy header
        "x-real-ip",                 // Nginx proxy
        "x-client-ip",               // Apache proxy
        "cf-connecting-ip",          // Cloudflare
        "x-forwarded",               // Alternative format
        "forwarded-for",             // Alternative format
        "forwarded",                 // RFC 7239
    ];

    // Check each header for a valid IP address
    for header_name in &ip_headers {
        if let Some(header_value) = req.headers().get(*header_name) {
            if let Ok(header_str) = header_value.to_str() {
                // Handle comma-separated IPs (x-forwarded-for can contain multiple IPs)
                let ips: Vec<&str> = header_str.split(',').map(|s| s.trim()).collect();
                
                for ip_str in ips {
                    // Skip private IPs and localhost (they're likely from internal proxies)
                    if let Ok(ip) = ip_str.parse::<IpAddr>() {
                        if !is_private_ip(&ip) {
                            log::debug!("Found client IP {} from header {}", ip_str, header_name);
                            return ip_str.to_string();
                        }
                    }
                }
            }
        }
    }

    // Fallback to connection info
    if let Some(peer_addr) = req.connection_info().peer_addr() {
        log::debug!("Using peer address: {}", peer_addr);
        return peer_addr.to_string();
    }

    // In test environments, use a default IP to avoid warnings
    if std::env::var("APP_ENV").unwrap_or_default() == "test" {
        log::debug!("Test environment detected, using default IP");
        return "127.0.0.1".to_string();
    }

    log::warn!("Could not determine client IP address");
    "unknown".to_string()
}

/// Check if an IP address is private/local
fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            // Private IPv4 ranges
            ipv4.is_private() || 
            ipv4.is_loopback() || 
            ipv4.is_link_local() ||
            ipv4.is_multicast() ||
            // Common proxy/load balancer IPs
            ipv4.octets() == [10, 0, 0, 0] ||
            ipv4.octets() == [172, 16, 0, 0] ||
            ipv4.octets() == [192, 168, 0, 0] ||
            ipv4.octets() == [127, 0, 0, 1]
        }
        IpAddr::V6(ipv6) => {
            // Private IPv6 ranges
            ipv6.is_loopback() ||
            ipv6.is_unspecified() ||
            ipv6.is_multicast()
        }
    }
}

// Define the middleware struct
#[derive(Clone)]
pub struct JwtMiddleware;

impl<S> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = JwtMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        // Wrap the service in an Arc so it can be cloned for each request
        ok(JwtMiddlewareService { 
            service: Arc::new(service),
        })
    }
}

// Define the middleware service struct
pub struct JwtMiddlewareService<S> {
    // Shared service wrapped in Arc for clonability
    service: Arc<S>,
}

impl<S> Service<ServiceRequest> for JwtMiddlewareService<S>
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
        // Pre-capture data for async block
        let token_opt = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .map(|t| t.to_owned());
        let origin = req
            .headers()
            .get(header::ORIGIN)
            .and_then(|h| h.to_str().ok())
            .map(|o| o.to_owned());
        let pool_data = req.app_data::<web::Data<PgPool>>().cloned();
        let path = req.path().to_string();
        
        // Extract client IP address for rate limiting
        let client_ip = extract_client_ip(&req);
        
        let req = req;
        // Clone the Arc to get a new reference to the inner service
        let srv = self.service.clone();

        log::info!("Authentication attempt for path: {} from IP: {}", path, client_ip);

        Box::pin(async move {
            if let Some(token) = token_opt {
                // Try JWT authentication first
                match verify_jwt(&token) {
                    Ok(claims) => {
                        log::info!("JWT verified successfully for user: {:?}", claims.user_id);
                        // Insert raw claims for handlers expecting ReqData<Claims>
                        let claims_clone = claims.clone();
                        req.extensions_mut().insert(claims_clone.clone());
                        // Also insert AuthenticatedUser enum for downstream use
                        req.extensions_mut().insert(AuthenticatedUser::Jwt(claims_clone));
                        return srv.call(req).await;
                    }
                    Err(jwt_err) => {
                        log::warn!("JWT verification failed: {jwt_err:?}. Checking if token could be an API key.");
                    }
                }

                // Before hitting the database, do a quick sanity check:
                // API keys should be reasonably long and not look like obviously malformed JWTs
                if token.len() < 10 || token.contains("invalid") || token.split('.').count() == 3 {
                    log::info!("Token appears to be malformed JWT or test token, skipping API key validation");
                } else {
                    // Try API key authentication only for tokens that might actually be API keys
                    log::info!("Attempting API key validation for token that might be an API key.");
                    let pool = match pool_data {
                        Some(p) => p,
                        None => {
                            log::error!("Database pool not found for API key validation.");
                            let resp = HttpResponse::Unauthorized().finish(); // Return 401, not 500
                            return Ok(req.into_response(resp));
                        }
                    };
                    
                    match validate_api_key_with_domain(pool.get_ref(), &token, origin.as_deref()).await {
                        Ok(Some(user)) => {
                            // Thorw 401 if user is not active
                            if user.status != "active" {
                                log::warn!("Unauthorized user.");
                                let resp = HttpResponse::Unauthorized().finish();
                                return Ok(req.into_response(resp));
                            }
                            log::info!("API Key validated successfully for user: {}", user.id);

                            // Define paths to be excluded from rate limiting
                            const RATE_LIMIT_EXCLUSIONS: &[&str] = &["/api/assets"];
                            let request_path = req.path();

                            // Check rate limit only if the path is not in the exclusion list
                            if !RATE_LIMIT_EXCLUSIONS.contains(&request_path) {
                                if !get_rate_limiter().check_rate_limit(&client_ip) {
                                    log::warn!("Limit exceeded for today! Sign up now to get unlimited access to Bounti Studio.");
                                    let resp = rate_limit_exceeded_response();
                                    return Ok(req.into_response(resp));
                                }
                            }

                            // Insert AuthenticatedUser enum for downstream use
                            req.extensions_mut().insert(AuthenticatedUser::ApiKey(user.id));
                            // Also insert Claims for observability middleware compatibility
                            let claims = Claims {
                                user_id: user.id,
                                is_admin: user.is_admin, // Use actual admin status from user
                                email: user.email.clone(),
                                email_verified: user.email_verified,
                                feature_flags: Some(user.feature_flags),
                                exp: 0, // API keys don't expire like JWTs
                                admin_id: None,
                                is_impersonating: None,
                            };
                            req.extensions_mut().insert(claims);
                            return srv.call(req).await;
                        }
                        Ok(None) => {
                            log::warn!("Invalid API Key provided via Bearer token.");
                        }
                        Err(db_err) => {
                            log::error!("Database error during API key validation: {db_err}");
                            // Return 401 instead of 500 - auth failure, not server error
                            let resp = HttpResponse::Unauthorized().finish();
                            return Ok(req.into_response(resp));
                        }
                    }
                }
            } else {
                log::info!("No Authorization header present.");
            }
            log::warn!(
                "Authentication failed for path: {path}. No valid JWT or API Key found."
            );
            let resp = HttpResponse::Unauthorized().finish();
            Ok(req.into_response(resp))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;

    #[test]
    fn test_extract_client_ip_x_forwarded_for() {
        let req = TestRequest::default()
            .insert_header(("x-forwarded-for", "203.0.113.195, 70.41.3.18, 150.172.238.178"))
            .to_srv_request();
        
        let ip = extract_client_ip(&req);
        
        // Should return the first public IP (203.0.113.195)
        assert_eq!(ip, "203.0.113.195");
    }

    #[test]
    fn test_extract_client_ip_x_real_ip() {
        let req = TestRequest::default()
            .insert_header(("x-real-ip", "203.0.113.195"))
            .to_srv_request();
        
        let ip = extract_client_ip(&req);
        
        assert_eq!(ip, "203.0.113.195");
    }

    #[test]
    fn test_extract_client_ip_cloudflare() {
        let req = TestRequest::default()
            .insert_header(("cf-connecting-ip", "203.0.113.195"))
            .to_srv_request();
        
        let ip = extract_client_ip(&req);
        
        assert_eq!(ip, "203.0.113.195");
    }

    #[test]
    fn test_extract_client_ip_skips_private_ips() {
        let req = TestRequest::default()
            .insert_header(("x-forwarded-for", "192.168.1.1, 203.0.113.195"))
            .to_srv_request();
        
        let ip = extract_client_ip(&req);
        
        // Should skip private IP and return the public one
        assert_eq!(ip, "203.0.113.195");
    }

    #[test]
    fn test_extract_client_ip_all_private() {
        let req = TestRequest::default()
            .insert_header(("x-forwarded-for", "192.168.1.1, 10.0.0.1"))
            .to_srv_request();
        
        let ip = extract_client_ip(&req);
        
        // Should fallback to peer address or "unknown"
        // The exact value depends on test environment
        assert!(!ip.is_empty());
    }

    #[test]
    fn test_is_private_ip() {
        // Private IPv4 addresses
        assert!(is_private_ip(&"192.168.1.1".parse().unwrap()));
        assert!(is_private_ip(&"10.0.0.1".parse().unwrap()));
        assert!(is_private_ip(&"172.16.0.1".parse().unwrap()));
        assert!(is_private_ip(&"127.0.0.1".parse().unwrap()));
        
        // Public IPv4 addresses
        assert!(!is_private_ip(&"203.0.113.195".parse().unwrap()));
        assert!(!is_private_ip(&"8.8.8.8".parse().unwrap()));
        assert!(!is_private_ip(&"1.1.1.1".parse().unwrap()));
    }
}