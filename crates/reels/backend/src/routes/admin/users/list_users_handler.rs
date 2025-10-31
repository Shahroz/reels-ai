//! Handler for listing users in the admin panel.

use crate::auth::tokens::Claims;
use crate::queries::admin::users::list_users_with_credits;
use crate::routes::admin::users::list_users_params::ListUsersParams;
use crate::routes::admin::users::list_users_response::{ListUsersResponse, EnrichedUserDto};
use crate::routes::error_response::ErrorResponse;
use actix_web::{web, HttpResponse, Responder};
use sqlx::PgPool;
use tracing::instrument;

#[utoipa::path(
    get,
    path = "/api/admin/users",
    tag = "Admin",
    params(
        ListUsersParams
    ),
    responses(
        (status = 200, description = "Successfully retrieved list of users", body = ListUsersResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::get("")]
#[instrument(skip(pool, auth_claims, params))]
pub async fn list_users_handler(
    pool: web::Data<PgPool>,
    auth_claims: Claims,
    params: web::Query<ListUsersParams>,
) -> impl Responder {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);
    let sort_by = params.sort_by.as_deref().unwrap_or("created_at");
    let sort_order = params.sort_order.as_deref().unwrap_or("desc");

    match list_users_with_credits::list_users_with_credits(
        &pool,
        page,
        limit,
        sort_by,
        sort_order,
        params.search.as_deref(),
        params.status.as_deref(),
    )
    .await
    {
        Ok((users, total_count)) => {
            // Check unlimited status for each user
            let mut enriched_users: Vec<EnrichedUserDto> = Vec::new();
            for user in users {
                let is_unlimited = crate::queries::unlimited_access::check_user_unlimited::check_user_unlimited(
                    &pool,
                    user.id,
                )
                .await
                .unwrap_or(false); // Default to false on error
                
                enriched_users.push(EnrichedUserDto {
                    id: user.id,
                    email: user.email,
                    status: user.status,
                    is_admin: user.is_admin,
                    feature_flags: user.feature_flags,
                    created_at: user.created_at,
                    credits_remaining: user.credits_remaining,
                    is_unlimited,
                });
            }
            
            HttpResponse::Ok().json(ListUsersResponse {
                items: enriched_users,
                total_count,
            })
        }
        Err(e) => {
            log::error!("Failed to list users: {e}");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve users.".to_string(),
            })
        }
    }
}