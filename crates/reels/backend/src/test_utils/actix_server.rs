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
use sqlx::{PgPool, postgres::PgPoolOptions};
use dotenvy::dotenv;
use std::env;
use actix_cors::Cors;
use actix_web::web::JsonConfig;
use std::sync::Once as StdOnce;

/// Ensures the test logger is only initialized once across all tests.
pub static TEST_LOGGER_INIT: StdOnce = StdOnce::new();

/// Sets up a test application factory and provides a fresh database pool.
///
/// This is the primary setup function for integration tests needing database access.
/// Each call creates fresh resources to avoid connection accumulation issues.
///
/// # Returns
/// A tuple containing:
/// - `App<impl ServiceFactory<...>>`: The configured Actix Web application factory.
///                                     Pass this to `actix_web::test::init_service()`.
/// - `PgPool`: A fresh database connection pool.
///
/// # Panics
/// - If critical environment variables are missing (e.g., `DATABASE_URL`).
/// - If the database URL safety checks fail.
/// - If AgentLoop state initialization fails.
pub async fn setup_test_app_and_pool() -> (
    App<
        impl ServiceFactory<
            ServiceRequest,
            Response = ServiceResponse<impl MessageBody>,
            Config = (),
            InitError = (),
            Error = Error,
        >,
    >,
    PgPool,
) {
    // Initialize logger (once per test suite run)
    TEST_LOGGER_INIT.call_once(|| {
        let default_log_filter = env::var("RUST_LOG")
            .unwrap_or_else(|_| "backend=warn,test_utils=info,actix_web=info,sqlx=warn".to_string());
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(&default_log_filter))
            .is_test(true) // Ensures logs are captured by the test runner
            .try_init()
            .ok(); // .ok() to ignore errors if already initialized
        log::info!("[Test Logger] Initialized. Default filter: '{default_log_filter}'. Override with RUST_LOG.");
    });

    // Load environment variables
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("FATAL: DATABASE_URL environment variable is not set. This is required for tests.");

    // Safety checks for the database URL
    let expected_test_port = env::var("POSTGRES_PORT").unwrap_or_else(|_| "5447".to_string());
    let expected_test_db_name_part = "localdatabase"; // A common pattern for test DB names

    if !database_url.contains(&format!(":{expected_test_port}")) ||
       !database_url.contains(expected_test_db_name_part) {
        panic!(
            "FATAL: DATABASE_URL ('{database_url}') does not seem to be a safe test URL. It should connect to port '{expected_test_port}' and DB name containing '{expected_test_db_name_part}'."
        );
    }

    // Create a fresh database pool for this test
    let pool = PgPoolOptions::new()
        .max_connections(20) // Reasonable limit for individual tests
        .min_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .idle_timeout(Some(std::time::Duration::from_secs(60)))
        .max_lifetime(Some(std::time::Duration::from_secs(300)))
        .test_before_acquire(true)
        .connect(&database_url)
        .await
        .expect("Failed to create test database pool.");

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
    let dub_service = crate::services::dub::DubService::from_env()
        .expect("Failed to create DubService for tests");

    // Create session manager for tests (same as in main.rs)
    let session_manager = std::sync::Arc::new(
        crate::services::session_manager::HybridSessionManager::new(pool.clone())
    );

    // Create Postmark client for tests (will use empty token, emails won't actually send)
    let postmark_client = std::sync::Arc::new(
        postmark::reqwest::PostmarkClient::builder()
            .server_token("") // Empty token for tests - emails won't send
            .build()
    );

    // Create the Actix Web application factory
    let app_factory = App::new()
        // Pass fresh resources as app_data
        .app_data(web::Data::new(pool.clone()))
        .app_data(web::Data::new(gcs_client.clone()))
        .app_data(web::Data::new(std::sync::Arc::new(gcs_client.clone()) as std::sync::Arc<dyn crate::services::gcs::gcs_operations::GCSOperations>))
        .app_data(web::Data::new(screenshot_service.clone()))
        .app_data(web::Data::new(postmark_client.clone()))
        .app_data(web::Data::new(agentloop_state_inner))
        .app_data(web::Data::from(std::sync::Arc::new(dub_service.clone()) as std::sync::Arc<dyn crate::services::dub::DubServiceTrait>))
        .app_data(web::Data::new(session_manager))
        
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

        // Add health check endpoint
        .service(routes::health::health_check)
        
        // Configure all other application routes
        .configure(routes::config);

    (app_factory, pool)
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
    let (app_factory, _pool) = setup_test_app_and_pool().await;
    app_factory
}
