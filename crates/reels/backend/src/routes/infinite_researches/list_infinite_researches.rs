//! Handler for listing all infinite research tasks for a user.

use crate::auth::tokens::Claims;
use crate::routes::error_response::ErrorResponse;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;

#[derive(serde::Serialize, utoipa::ToSchema, sqlx::FromRow, Debug)]
pub struct InfiniteResearchListItem {
    #[schema(value_type = String, format = "uuid")]
    pub id: uuid::Uuid,
    #[schema(value_type = String, format = "uuid")]
    pub user_id: uuid::Uuid,
    pub name: std::string::String,
    pub prompt: std::string::String,
    pub cron_schedule: std::string::String,
    pub is_enabled: bool,
    #[schema(nullable = true)]
    pub scheduler_job_name: std::option::Option<std::string::String>,
    #[schema(value_type = String, format = "date-time")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(value_type = String, format = "date-time")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[schema(nullable = true, example = "completed")]
    pub last_execution_status: std::option::Option<std::string::String>,
}

#[utoipa::path(
    get,
    path = "/api/infinite-researches",
    tag = "Infinite Research",
    responses(
        (status = 200, description = "List of infinite research tasks", body = Vec<InfiniteResearchListItem>),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(("user_auth" = []))
)]
#[get("")]
pub async fn list_infinite_researches(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    let user_id = claims.user_id;

    let query = sqlx::query_as!(
        InfiniteResearchListItem,
        r#"
        WITH latest_executions AS (
            SELECT
                infinite_research_id,
                status,
                ROW_NUMBER() OVER(PARTITION BY infinite_research_id ORDER BY started_at DESC) as rn
            FROM infinite_research_executions
        )
        SELECT
            ir.id,
            ir.user_id,
            ir.name,
            ir.prompt,
            ir.cron_schedule,
            ir.is_enabled,
            ir.scheduler_job_name,
            ir.created_at,
            ir.updated_at,
            le.status as "last_execution_status: _"
        FROM infinite_researches ir
        LEFT JOIN latest_executions le ON ir.id = le.infinite_research_id AND le.rn = 1
        WHERE ir.user_id = $1
        ORDER BY ir.updated_at DESC
        "#,
        user_id
    );

    match query.fetch_all(pool.get_ref()).await {
        Ok(researches) => HttpResponse::Ok().json(researches),
        Err(e) => {
            log::error!("Failed to list infinite researches for user {user_id}: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve research tasks.".into(),
            })
        }
    }
}
