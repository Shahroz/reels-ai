//! Handler for listing all organizations in the admin panel.
//!
//! This endpoint allows administrators to view all organizations in the system with
//! enriched data including owner email and member count. Supports search, filtering,
//! sorting, and pagination for building comprehensive admin interfaces.

#[utoipa::path(
    get,
    path = "/api/admin/organizations",
    tag = "Admin",
    params(
        crate::routes::admin::organizations::list_all_organizations_params::ListAllOrganizationsParams
    ),
    responses(
        (status = 200, description = "Successfully retrieved organizations", body = crate::routes::admin::organizations::list_all_organizations_response::ListAllOrganizationsResponse),
        (status = 401, description = "Unauthorized - user is not an admin", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::get("")]
#[tracing::instrument(skip(pool, auth_claims, params))]
pub async fn list_all_organizations_handler(
    pool: actix_web::web::Data<sqlx::PgPool>,
    auth_claims: crate::auth::tokens::Claims,
    params: actix_web::web::Query<crate::routes::admin::organizations::list_all_organizations_params::ListAllOrganizationsParams>,
) -> impl actix_web::Responder {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20);
    let sort_by = params.sort_by.as_deref().unwrap_or("created_at");
    let sort_order = params.sort_order.as_deref().unwrap_or("desc");

    match crate::queries::admin::organizations::list_organizations_with_credits::list_organizations_with_credits(
        &pool,
        page,
        limit,
        params.search.as_deref(),
        sort_by,
        sort_order,
    )
    .await
    {
        Ok((orgs, total_count)) => {
            let items: Vec<crate::routes::admin::organizations::list_all_organizations_response::EnrichedOrganizationDto> = orgs
                .into_iter()
                .map(|org| crate::routes::admin::organizations::list_all_organizations_response::EnrichedOrganizationDto {
                    id: org.id,
                    name: org.name,
                    owner_user_id: org.owner_user_id,
                    owner_email: org.owner_email,
                    member_count: org.member_count,
                    created_at: org.created_at,
                    updated_at: org.updated_at,
                    credits_remaining: org.credits_remaining,
                })
                .collect();

            actix_web::HttpResponse::Ok().json(crate::routes::admin::organizations::list_all_organizations_response::ListAllOrganizationsResponse {
                items,
                total_count,
                page,
                limit,
            })
        }
        Err(e) => {
            log::error!(
                "Failed to list organizations for admin user {}: page={}, limit={}, search={:?}, sort_by={}, sort_order={}, error: {}",
                auth_claims.user_id,
                page,
                limit,
                params.search,
                sort_by,
                sort_order,
                e
            );
            actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: String::from("Failed to retrieve organizations. Please try again or contact support if the issue persists."),
            })
        }
    }
}
