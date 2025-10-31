use axum::{
    routing::{get, post, put, delete},
    Router,
};
use sqlx::PgPool;

use crate::api::handlers::research_workflows::*;

pub mod handlers;

pub fn create_router(pool: PgPool) -> Router {
    Router::new()
        .route("/api/research-workflows", get(get_research_workflows))
        .route("/api/research-workflows/:id", get(get_research_workflow))
        .route("/api/research-workflows", post(create_research_workflow))
        .route("/api/research-workflows/:id", put(update_research_workflow))
        .route("/api/research-workflows/:id", delete(delete_research_workflow))
        .with_state(pool)
}