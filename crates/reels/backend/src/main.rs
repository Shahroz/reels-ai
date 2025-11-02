mod app_constants; // Added for constants module
mod openapi; // Added for OpenAPI documentation
mod routes;
mod services;
mod utils;
mod zyte; // Corrected: Removed duplicate 'mod clone' // Added to declare the module
mod webflow; // Added webflow module
mod middleware; // Added middleware module
// mod billing_service; // Removed - billing service is in services module

// use crate::routes::health; // Route folder deleted
// Still needed if health_check registered directly
pub mod app;
pub mod gcp_auth;
pub mod agent_tools;
pub mod llm_support;
pub mod query_parser;
pub mod test_utils;
use crate::services::gcs::gcs_client::GCSClient;

// OpenAPI imports
use crate::openapi::ApiDoc;
use utoipa::OpenApi;

use actix_cors::Cors;
use actix_web::{middleware as actix_middleware, web, App, HttpServer};
use dotenvy::dotenv;

use actix_web::web::JsonConfig;
// Added for CORS
use std::env;
 // Added for GLOBAL_POOL
 // Added for GLOBAL_POOL type

use utoipa_swagger_ui::SwaggerUi;
// Added for environment variables

async fn setup_server() -> std::io::Result<()> {
   // Initialize dotenv to load .env file
   dotenv().ok();
   // Database pool creation removed - db module deleted
   // let pool = crate::db::create_pool::create_pool()
   //     .await
   //     .expect("Failed to create database pool.");
   // Instantiate GCS Client
   let gcs_client = GCSClient::new();

    // Cloud Tasks Client removed - gcp module deleted
    // let tasks_client = CloudTasksClient::new()
    //     .await
    //     .expect("Failed to create CloudTasksClient");

    // Instantiate Screenshot Service
    let screenshot_config = crate::services::screenshot::screenshot_config::ScreenshotConfig::from_env();
    let screenshot_service = crate::services::screenshot::service_factory::create_screenshot_service(&screenshot_config)
        .expect("Failed to create screenshot service");

    // --- Reels Custom Tools Configuration for AgentLoop ---
    // This now provides both definitions and handlers to AgentLoop.
    // --- AgentLoop State Initialization ---
    log::info!("Initializing AgentLoop state...");
    // Note: agentloop::app_setup::configure_app expects its own config loaded via env vars.
    // Ensure necessary env vars for agentloop (DATABASE_URL, etc. if needed by agentloop) are set.
    // THE `configure_app` SIGNATURE IN AGENTLOOP WOULD NEED TO BE MODIFIED
    // TO ACCEPT `narrativ_external_tools_config`. For example:
    // `agentloop::app_setup::configure_app(Some(narrativ_external_tools_config)).await`
    let agentloop_state = crate::services::agent_service::create_agentloop_state()
        .await
        .expect("Failed to initialize AgentLoop state");
    // log::info!("AgentLoop state wrapped in web::Data.");

    // The old `custom_tools_data` (HashMap of handlers) is now superseded by passing
    // the full ExternalToolsConfig to AgentLoop's setup. AgentLoop's AppState
    // will hold the merged handlers.

    // --- OpenAPI Documentation Setup ---
    let openapi = ApiDoc::openapi();

    // Determine APP_ENV (default to "production" if not set)
    let app_env = env::var("APP_ENV").unwrap_or_else(|_| "production".to_string());
    log::info!("APP_ENV is set to: {app_env}"); // Log the environment

    // Determine Port from PORT env var, default to 8080
    let port = match env::var("PORT") {
        Ok(val) => match val.parse::<u16>() {
            Ok(p) => p,
            Err(_) => {
                log::warn!("Invalid PORT value '{val}' provided, defaulting to 8080.");
                8080
            }
        },
        Err(_) => {
            log::info!("PORT environment variable not set, defaulting to 8080.");
            8080
        }
    };

    log::info!("Attempting to bind server to 0.0.0.0:{port}");

    HttpServer::new(move || {
        // Define CORS based on environment
        let cors = if app_env == "development" {
            // Permissive CORS for development
            log::info!("Applying permissive CORS for development mode.");
            Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .supports_credentials()
                .max_age(3600)
        } else {
            // Production CORS: Allow webhooks while maintaining security
            log::info!("Applying production CORS with webhook support.");
            Cors::default()
                .allow_any_origin() // Needed for Stripe and other webhooks
                .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS", "HEAD"]) // Allow PATCH for /api/assets/attach
                .allow_any_header() // Needed for webhook signature headers
                .max_age(3600)
                // Note: Webhooks don't use credentials, so we don't need supports_credentials()
        };

        {
            let app = App::new()
                // .app_data(web::Data::new(pool.clone())) // Database pool removed - db module deleted
                .app_data(web::Data::new(gcs_client.clone()))
                // .app_data(web::Data::new(tasks_client.clone())) // Cloud Tasks removed - gcp module deleted
                .app_data(web::Data::new(screenshot_service.clone()))
                .app_data(agentloop_state.clone())
                .app_data(web::Data::new(std::sync::Arc::new(gcs_client.clone()) as std::sync::Arc<dyn crate::services::gcs::gcs_operations::GCSOperations>))
                .app_data(agentloop_state.clone()) // Add agentloop state
                // File size limits:
                // - File API (for videos): up to 2GB supported by Gemini
                // - Base64 uploads (for documents/images): 100MB raw = ~133MB encoded
                // - Current limit: 200MB to accommodate both video and document uploads
                // TODO: Consider multipart uploads to avoid base64 inflation for non-video files
                .app_data(JsonConfig::default().limit(2000 * 1024 * 1024)) // 2000MB
                // Apply middleware in correct order (last applied = first executed)
                .wrap(actix_middleware::Logger::new(
                    r#"%a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#
                )) // Clean HTTP request logging
                .wrap(cors); // Apply CORS middleware

            let app = app;

            // Session manager removed - db module deleted
            // let session_manager = std::sync::Arc::new(
            //     crate::services::session_manager::HybridSessionManager::new(pool.clone())
            // );
            
            // Background cleanup task removed - session manager removed
            // crate::services::session_manager::HybridSessionManager::start_cleanup_task(session_manager.clone());

            app
                // .app_data(web::Data::new(session_manager)) // Session manager removed
                // --- OpenAPI Swagger UI Service ---
                .service(
                    SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
                )
                // --- Well-Known Endpoints (must be FIRST, before catch-all) ---
                // .service(crate::routes::apple_app_site_association::apple_app_site_association) // Route folder deleted
                // --- API Routes Configuration ---
                // .service(health::health_check) // Route folder deleted
                .configure(crate::routes::config)
                // Use the config function from the routes module
        }
        })
        .bind(("0.0.0.0", port))?
        .run()
        .await
}

fn main() -> std::io::Result<()> {
    // Initialize dotenv to load .env file
    dotenv().ok();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime");

    runtime.block_on(async {
        // Initialize dotenv to load .env file
        dotenv().ok();
        // Determine APP_ENV early for logger setup
        let app_env_for_logger = env::var("APP_ENV").unwrap_or_else(|_| "production".to_string());

        // Initialize env_logger based on the environment
        if app_env_for_logger == "development" || app_env_for_logger == "dev" {
            // In development/dev, use clean logging without colors for better debugging
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
                .format_timestamp_secs()
                .format_module_path(false)
                .format_target(true)
                .init();
            log::info!("Development environment detected. Initializing clean env_logger with level 'info'.");
        } else {
            // In production, use clean logging without colors and simplified format
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
                .format_timestamp_secs()
                .format_module_path(false) 
                .format_target(true)
                .init();
            log::info!("Production environment detected. Initializing clean env_logger.");
        }


        setup_server().await?;

        Ok(())
    })
}
