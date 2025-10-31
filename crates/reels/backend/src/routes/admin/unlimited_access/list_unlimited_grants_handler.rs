//! Handler for listing unlimited access grants via admin endpoint.
//!
//! This endpoint allows administrators to view all unlimited access grants
//! with optional filtering and pagination. Can include revoked grants or
//! show only active grants. Only admin users can access this endpoint.

#[utoipa::path(
    get,
    path = "/api/admin/unlimited-access/grants",
    tag = "Admin",
    params(
        crate::routes::admin::unlimited_access::list_unlimited_grants_params::ListUnlimitedGrantsParams
    ),
    responses(
        (status = 200, description = "Successfully retrieved unlimited access grants", body = crate::routes::admin::unlimited_access::list_unlimited_grants_response::ListUnlimitedGrantsResponse),
        (status = 401, description = "Unauthorized", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::get("/grants")]
#[tracing::instrument(skip(pool, _auth_claims, params))]
pub async fn list_unlimited_grants_handler(
    pool: actix_web::web::Data<sqlx::PgPool>,
    _auth_claims: crate::auth::tokens::Claims,
    params: actix_web::web::Query<crate::routes::admin::unlimited_access::list_unlimited_grants_params::ListUnlimitedGrantsParams>,
) -> impl actix_web::Responder {
    // Cap limit at 100 to prevent DoS
    const MAX_PAGE_SIZE: i64 = 100;
    let limit = std::cmp::min(
        params.limit.unwrap_or(20),
        MAX_PAGE_SIZE
    );
    let offset = params.offset.unwrap_or(0);
    let include_revoked = params.include_revoked.unwrap_or(false);
    
    // Get total count (not just page size)
    let total: i64 = if include_revoked {
        match sqlx::query_scalar!(
            "SELECT COUNT(*) FROM unlimited_access_grants"
        )
        .fetch_one(pool.get_ref())
        .await
        {
            Ok(Some(count)) => count,
            Ok(None) => 0,
            Err(e) => {
                log::error!("Failed to count unlimited access grants: {}", e);
                return actix_web::HttpResponse::InternalServerError().json(
                    crate::routes::error_response::ErrorResponse {
                        error: "Failed to count unlimited access grants.".to_string(),
                    },
                );
            }
        }
    } else {
        match sqlx::query_scalar!(
            "SELECT COUNT(*) FROM unlimited_access_grants 
             WHERE revoked_at IS NULL AND (expires_at IS NULL OR expires_at > NOW())"
        )
        .fetch_one(pool.get_ref())
        .await
        {
            Ok(Some(count)) => count,
            Ok(None) => 0,
            Err(e) => {
                log::error!("Failed to count active unlimited access grants: {}", e);
                return actix_web::HttpResponse::InternalServerError().json(
                    crate::routes::error_response::ErrorResponse {
                        error: "Failed to count active unlimited access grants.".to_string(),
                    },
                );
            }
        }
    };
    
    // Get grants from database
    let grants = match crate::queries::unlimited_access::list_all_grants::list_all_grants(
        pool.get_ref(),
        include_revoked,
        limit,
        offset,
    )
    .await
    {
        Ok(grants) => grants,
        Err(e) => {
            log::error!("Failed to list unlimited access grants: {}", e);
            return actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: "Failed to retrieve unlimited access grants.".to_string(),
                },
            );
        }
    };
    
    actix_web::HttpResponse::Ok().json(
        crate::routes::admin::unlimited_access::list_unlimited_grants_response::ListUnlimitedGrantsResponse {
            grants,
            total: total as usize,
        },
    )
}

