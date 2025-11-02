//! Test server setup utilities for Actix Web integration tests
//!
//! This module provides functions to create fully configured Actix Web test applications
//! with database connections, middleware, and all necessary dependencies for integration testing.
//!
//! # Usage
//!
//! For tests that need both the app and a database pool:
//! ```ignore
//! use actix_web::test;
//! use narrativ::test_utils::actix_server::setup_test_app_and_pool;
//! use narrativ::test_utils::helpers::TestUser;
//!
//! #[tokio::test]
//! async fn test_endpoint_with_db() {
//!     let (app_factory, pool) = setup_test_app_and_pool().await;
//!     let app = test::init_service(app_factory).await;
//!
//!     // Create a test user
//!     let test_user = TestUser::new(pool.clone()).await.expect("Failed to create test user");
//!
//!     // Use app for HTTP requests
//!     let req = test::TestRequest::get()
//!         .uri("/api/some_protected_endpoint")
//!         .insert_header(("Authorization", test_user.auth_header()))
//!         .to_request();
//!     let resp = test::call_service(&app, req).await;
//!     assert!(resp.status().is_success());
//!
//!     // Clean up
//!     test_user.cleanup().await.expect("Failed to cleanup test user");
//! }
//! ```
//!
//! For tests that only need the app (e.g., public endpoints):
//! ```ignore
//! use actix_web::test;
//! use narrativ::test_utils::actix_server::setup_test_app_service;
//!
//! #[tokio::test]
//! async fn test_public_endpoint_only() {
//!     let app_factory = setup_test_app_service().await;
//!     let app = test::init_service(app_factory).await;
//!
//!     let req = test::TestRequest::get().uri("/health").to_request();
//!     let resp = test::call_service(&app, req).await;
//!     assert!(resp.status().is_success());
//! }
//! ```
//!
//! # Environment Variables Required
//!
//! - `DATABASE_URL`: PostgreSQL connection URL for the test database.
//! - `JWT_SECRET`: Secret key for JWT token generation (if tests involve auth).
//! - `RUST_LOG` (optional): Log level configuration (e.g., "info,sqlx=warn").
//! - `POSTGRES_PORT` (optional): Expected test database port (defaults to 5447 for safety checks).
//!
//! # Key Features
//!
//! 1.  **Fresh Resources Per Test**: Each test gets its own database pool and AgentLoop state
//!     to avoid resource contention and connection accumulation issues.
//! 2.  **Test Isolation**: Tests are independent and clean up their own resources.
//!
//! # Safety Features
//!
//! The setup includes safety checks to prevent accidental connection to non-test databases:
//! - Validates `DATABASE_URL` contains the expected test port (default 5447).
//! - Validates `DATABASE_URL` contains "localdatabase" in the name.
//! - Panics if these checks fail.

use actix_web::{
    web, App, Error,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    body::MessageBody,
    middleware as actix_middleware_log,
};
use crate::{
    services::gcs::gcs_client::GCSClient,
    routes,
};
// sqlx removed - no database interaction
use dotenvy::dotenv;
use std::env;
use actix_cors::Cors;
use actix_web::web::JsonConfig;
use std::sync::Once as StdOnce;

/// Ensures the test logger is only initialized once across all tests.
pub static TEST_LOGGER_INIT: StdOnce = StdOnce::new();

/// Sets up a test application factory.
///
/// Note: Database functionality removed - sqlx dependency removed.
/// This function no longer provides a database pool.
///
/// # Returns
/// `App<impl ServiceFactory<...>>`: The configured Actix Web application factory.
///                                   Pass this to `actix_web::test::init_service()`.
///
/// # Panics
/// - If AgentLoop state initialization fails.
pub async fn setup_test_app_and_pool() -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = Error,
    >,
> {
    // Initialize logger (once per test suite run)
    TEST_LOGGER_INIT.call_once(|| {
        let default_log_filter = env::var("RUST_LOG")
            .unwrap_or_else(|_| "backend=warn,test_utils=info,actix_web=info".to_string());
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&default_log_filter))
            .is_test(true) // Ensures logs are captured by the test runner
            .try_init()
            .ok(); // .ok() to ignore errors if already initialized
        log::info!("[Test Logger] Initialized. Default filter: '{default_log_filter}'. Override with RUST_LOG.");
    });

    // Load environment variables
    dotenv().ok();

    // Database functionality removed - sqlx dependency removed

    // Initialize other application dependencies
   let gcs_client = GCSClient::new(); // Assuming GCSClient::new() is relatively lightweight
   
   // Use screenshot service (will automatically be mock in test environment)
   let screenshot_config = crate::services::screenshot::screenshot_config::ScreenshotConfig::for_tests();
   let screenshot_service = crate::services::screenshot::service_factory::create_screenshot_service(&screenshot_config)
       .expect("Failed to create screenshot service for tests");

   // Initialize AgentLoop state fresh for each test
    let agentloop_state_inner = match agentloop::app_setup::configure_app(None, None).await {
       Ok(state) => state,
       Err(e) => {
           log::error!("FATAL: Failed to initialize AgentLoop state for test: {e}");
            panic!("Failed to initialize AgentLoop state for test: {e}");
        }
    };

    // Initialize DubService for tests (will use disabled mode if env vars not set)
    // DubService removed - services::dub module deleted
    // let dub_service = crate::services::dub::DubService::from_env()
    //     .expect("Failed to create DubService for tests");

    // Session manager - database functionality removed
    // let session_manager = std::sync::Arc::new(
    //     crate::services::session_manager::HybridSessionManager::new(pool.clone())
    // );

    // Create Postmark client for tests (will use empty token, emails won't actually send)
    let postmark_client = std::sync::Arc::new(
        postmark::reqwest::PostmarkClient::builder()
            .server_token("") // Empty token for tests - emails won't send
            .build()
    );

    // Create the Actix Web application factory
    let app_factory = App::new()
        // Pass fresh resources as app_data
        .app_data(web::Data::new(gcs_client.clone()))
        .app_data(web::Data::new(std::sync::Arc::new(gcs_client.clone()) as std::sync::Arc<dyn crate::services::gcs::gcs_operations::GCSOperations>))
        .app_data(web::Data::new(screenshot_service.clone()))
        .app_data(web::Data::new(postmark_client.clone()))
        .app_data(web::Data::new(agentloop_state_inner))
        // DubService removed - services::dub module deleted
        // .app_data(web::Data::from(std::sync::Arc::new(dub_service.clone()) as std::sync::Arc<dyn crate::services::dub::DubServiceTrait>))
        // Session manager removed - database functionality removed
        
        // Add JSON configuration for handling request bodies
        .app_data(JsonConfig::default().limit(4096)) // 4KB limit for test payloads

        // Add CORS middleware (permissive for tests)
        .wrap(
            Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .supports_credentials()
        )

        // Add request logging middleware
        .wrap(actix_middleware_log::Logger::default())

        // Add health check endpoint - route folder deleted
        // .service(routes::health::health_check)
        
        // Configure all other application routes
        .configure(routes::config);

    app_factory
}

/// Sets up a test application factory without a database pool.
///
/// Use this for tests that only need to test HTTP endpoints without database access.
///
/// # Returns
/// A configured Actix Web application factory. Pass this to `actix_web::test::init_service()`.
pub async fn setup_test_app_service() -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = Error,
    >,
> {
    setup_test_app_and_pool().await
}
