use actix_web::{web, HttpResponse, get, Responder};
use sqlx::PgPool;

use crate::auth::tokens::Claims;

/// Get current user subscription details from JWT claims
#[utoipa::path(
    get,
    path = "/api/users/subscriptions/me",
    tag = "Users",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Current user subscription details retrieved successfully", body = crate::db::user_subscription::UserSubscription),
        (status = 401, description = "Unauthorized - Invalid or missing JWT"),
        (status = 404, description = "User subscription not found"),
        (status = 500, description = "Internal server error")
    ),
)]
#[get("/subscriptions/me")]
pub async fn get_current_user_subscription(
    pool: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    let user_id = claims.user_id;
    log::info!("Getting current user subscription for user_id: {}", user_id);
    use crate::queries::user_subscription::get_user_subscription_by_user_id;

    match get_user_subscription_by_user_id(&pool, user_id).await {
        Ok(Some(subscription)) => HttpResponse::Ok().json(subscription),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "User subscription not found"
        })),
        Err(e) => {
            log::error!("Failed to get current user subscription: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to get current user subscription: {}", e)
            }))
        }
    }
}
