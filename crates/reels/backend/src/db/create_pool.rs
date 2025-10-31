use std::env;
use sqlx::{Error, PgPool};

/// Creates a PostgreSQL connection pool.
///
/// Loads the DATABASE_URL from the environment (using .env file if present)
/// and establishes a connection pool with a maximum of 5 connections.
pub async fn create_pool() -> Result<PgPool, Error> {
    // Load environment variables from .env file if it exists.
    dotenvy::dotenv().ok();

    // Read the database URL from the environment variables.
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in the environment or .env file");

    // Create the connection pool.
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
}