//! Entry point for the AgentLoop Actix-web server application.
//!
//! Initializes application state including configuration, sets up routing,
//! applies middleware, spawns background tasks, and starts the HTTP server.
//! Adheres to Rust coding guidelines for structure and path usage.
//! Manages the main application lifecycle.

// Note: Using fully qualified paths for all imports and types as per guidelines.
pub mod auth;
// pub mod background;
pub mod config;
pub mod conversation;
pub mod handlers;
pub mod session;
pub mod state;
pub mod assets; // Import the assets module
pub mod tools;
pub mod types;
pub mod websocket;
pub mod utils;
pub mod evaluator;
pub mod openapi;
// -- Embedded Frontend Assets (mirroring narrativ backend) --
use rust_embed::RustEmbed;
use mime_guess::from_path;

#[derive(RustEmbed)]
#[folder = "frontend/dist"]
struct Asset;

/// Serves an embedded file, falling back to index.html for SPA routing.
fn handle_embedded_file(path: &str) -> actix_web::HttpResponse {
    match Asset::get(path) {
        Some(content) => {
            let content_type = from_path(path).first_or_octet_stream();
            actix_web::HttpResponse::Ok()
                .content_type(content_type.as_ref())
                .body(content.data.into_owned())
        }
        None => handle_embedded_file("index.html"),
    }
}

/// Route for serving index.html from embedded assets
#[actix_web::get("/")]
async fn index() -> actix_web::HttpResponse {
    handle_embedded_file("index.html")
}

/// Route for serving any other static file or SPA fallback
#[actix_web::get("/{_:.*}")]
async fn dist(path: actix_web::web::Path<String>) -> actix_web::HttpResponse {
    handle_embedded_file(path.as_str())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok(); // Load .env f
    // Initialize logger. Expects RUST_LOG environment variable (e.g., RUST_LOG=info).
    // Requires adding `env_logger = "0.10"` or similar to Cargo.toml dependencies.
    env_logger::init();

    // Load configuration from environment variables using the dedicated function.
    // Handle potential errors during loading.
    let config = match crate::config::load_env_config::load_env_config() {
        std::result::Result::Ok(cfg) => cfg,
        std::result::Result::Err(e) => {
            // Use log::error! for consistency with logger initialization.
            log::error!("FATAL: Failed to load configuration: {}", e);
            return std::result::Result::Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Configuration loading failed: {}", e),
            ));
        }
    };

   // Initialize application state using the dedicated function and loaded config.
   // Handle potential errors during state initialization.
   let app_state = match crate::state::init_app_state::init_app_state(config.clone(), None, None).await {
       std::result::Result::Ok(state) => state,
       std::result::Result::Err(e) => {
           log::error!("FATAL: Failed to initialize application state: {}", e);
            return std::result::Result::Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("State initialization failed: {}", e),
            ));
        }
    };

    // Wrap AppState in Data for sharing across threads/handlers.
    let app_data = actix_web::web::Data::new(app_state);

    // Spawn background tasks using tokio::spawn.
    // Clone AppData for each task to ensure ownership is handled correctly.
    // These tasks are crucial for picking up Pending sessions (e.g., after UserInput)
    // and handling timeouts.
    // log::info!("Spawning background tasks...");
    // tokio::spawn(crate::background::tasks::background_evaluator_task(app_data.clone()));
    // log::info!("Spawned background evaluator task.");
    // tokio::spawn(crate::background::tasks::timeout_task(app_data.clone())); // Assuming timeout_task is still relevant
    // log::info!("Spawned timeout task.");

    // Configure and start the Actix-web HTTP server.
    let server_address = config.server_address.clone(); // Clone address for the closure.
    log::info!("Starting HTTP server on http://{}", &server_address);

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            // Share application state with handlers.
            .app_data(app_data.clone())
            // Add basic logging middleware from Actix Web.
            .wrap(actix_web::middleware::Logger::default())            // Add permissive CORS middleware (ALLOWS ALL ORIGINS - use carefully!)
            .wrap(actix_cors::Cors::permissive())
            // Apply custom authentication middleware.
            .wrap(crate::auth::middleware::BearerAuth)
            // Configure routes using the dedicated function.
            // Configure API routes first.
            .configure(crate::config::routes::configure_routes)
            // Embedded SPA assets served via RustEmbed handlers
            .service(index)
            .service(dist)
    })
        .bind(&server_address)? // Bind to the specified address and port.
        .run() // Run the server.
        .await // Await server termination.
}
