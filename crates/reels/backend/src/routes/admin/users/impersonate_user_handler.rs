//! Handler for initiating user impersonation.
//!
//! Allows an administrator to start an impersonation session for a specified user.
//! The administrator must not be currently impersonating another user. Admin-to-admin
//! impersonation is permitted, allowing admins to test the admin experience as other admins.
//!
//! Revision History:
//! - 2025-10-10: Removed use statements, clarified admin-to-admin impersonation is allowed.

const JWT_EXPIRATION_HOURS: i64 = 24; // Standard JWT expiration

#[utoipa::path(
    post,
    path = "/api/admin/users/{user_id}/impersonate",
    tag = "Admin",
    params(
        ("user_id" = uuid::Uuid, Path, description = "The ID of the user to impersonate.")
    ),
    responses(
        (status = 200, description = "Impersonation successful", body = crate::routes::admin::users::impersonate_user_response::ImpersonateUserResponse),
        (status = 400, description = "Bad Request (e.g., admin already impersonating)", body = crate::routes::error_response::ErrorResponse),
        (status = 401, description = "Unauthorized (e.g., requester is not an admin)", body = crate::routes::error_response::ErrorResponse),
        (status = 403, description = "Forbidden - admin-to-admin impersonation is permitted", body = crate::routes::error_response::ErrorResponse),
        (status = 404, description = "Target user not found", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::post("/{user_id}/impersonate")]
#[tracing::instrument(skip(pool, auth_claims, target_user_id_path))]
pub async fn impersonate_user_handler(
    pool: actix_web::web::Data<sqlx::PgPool>,
    auth_claims: crate::auth::tokens::Claims, // Extracted by JwtMiddleware
    target_user_id_path: actix_web::web::Path<uuid::Uuid>,
) -> impl actix_web::Responder {
    // 1. Verify requester is not already impersonating.
    if auth_claims.is_impersonating.unwrap_or(false) {
        return actix_web::HttpResponse::BadRequest().json(crate::routes::error_response::ErrorResponse {
            error: String::from("Cannot start a new impersonation session while already impersonating."),
        });
    }

    let original_admin_id = auth_claims.user_id;
    let target_user_id = target_user_id_path.into_inner();

    // Prevent admin from impersonating themselves
    if original_admin_id == target_user_id {
        return actix_web::HttpResponse::BadRequest().json(crate::routes::error_response::ErrorResponse {
            error: String::from("Admin cannot impersonate themselves."),
        });
    }

    // 2. Fetch target user details.
    let target_user = match crate::db::users::find_user_by_id(&pool, target_user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return actix_web::HttpResponse::NotFound().json(crate::routes::error_response::ErrorResponse {
                error: std::format!("Target user with ID {target_user_id} not found."),
            });
        }
        Err(e) => {
            log::error!("Failed to fetch target user {target_user_id}: {e}");
            return actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: String::from("Failed to retrieve target user details."),
            });
        }
    };

    // 3. Generate new JWT for impersonation.
    let now = chrono::Utc::now();
    let exp = (now + chrono::Duration::hours(JWT_EXPIRATION_HOURS)).timestamp() as u64;

    let impersonation_claims = crate::auth::tokens::Claims {
        user_id: target_user.id,                 // Impersonated user's ID
        is_admin: target_user.is_admin,          // Impersonated user's admin status
        email: target_user.email.clone(),        // Impersonated user's email
        email_verified: target_user.email_verified, // Impersonated user's email verification status
        exp,                                     // Standard expiration
        admin_id: Some(original_admin_id),       // Original admin's ID
        is_impersonating: Some(true),            // Flag indicating impersonation
        feature_flags: Some(target_user.feature_flags.clone()),
    };

    let token = match crate::auth::tokens::create_jwt(&impersonation_claims) {
        Ok(t) => t,
        Err(e) => {
            log::error!("Failed to create impersonation JWT: {e}");
            return actix_web::HttpResponse::InternalServerError().json(crate::routes::error_response::ErrorResponse {
                error: String::from("Failed to generate impersonation token."),
            });
        }
    };

    // 4. Return new JWT and target user's PublicUser object.
    let public_target_user: crate::db::users::PublicUser = target_user.into();
    actix_web::HttpResponse::Ok().json(crate::routes::admin::users::impersonate_user_response::ImpersonateUserResponse {
        token,
        user: public_target_user,
    })
}
