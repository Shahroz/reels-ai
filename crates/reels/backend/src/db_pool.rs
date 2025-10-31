use dotenvy::dotenv;
use std::env;
use actix_web::web;
use sqlx::postgres::PgPoolOptions;
use once_cell::sync::Lazy;
use sqlx::PgPool;

// --- Global Database Pool ---
// As per request: "although it is not beautiful I need to create a global postgres pool (wrapped in Data - same as actix)"
// This global pool is initialized on its first access.
// It creates a temporary Tokio runtime to block on the asynchronous pool creation.
// It's crucial that dotenvy::dotenv().ok() (called in main() or setup_server()) has run
// before this pool is first accessed, so environment variables like DATABASE_URL are loaded.
/// Global database pool initialized on first access.
pub static GLOBAL_POOL: Lazy<web::Data<PgPool>> = Lazy::new(|| {
    log::info!("Initializing GLOBAL_POOL...");
    // Load environment variables in case .env has not been loaded yet.
    dotenv().ok();
    // Read the database URL from the environment.
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in the environment or .env file");
    // Create a lazily connecting pool; connections are established on first use.
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(&database_url)
        .expect("Failed to create global database pool");
    log::info!("GLOBAL_POOL initialized successfully.");
    web::Data::new(pool)
});