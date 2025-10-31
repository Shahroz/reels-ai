//! Handler for password reset initiation.
//!
//! Sends a reset email if the user exists.
use crate::routes::auth::password_reset_request::PasswordResetRequest;
use tracing::instrument;

#[utoipa::path(
    post,
    path = "/auth/password-reset",
    tag = "Auth",
    request_body = PasswordResetRequest,
    responses(
        (status = 200, description = "Password reset initiated"),
        (status = 500, description = "Internal server error")
    )
)]
#[actix_web::post("/password-reset")]
#[instrument(skip(pool, req))]
pub async fn password_reset(
    pool: actix_web::web::Data<sqlx::PgPool>,
    postmark_client: actix_web::web::Data<std::sync::Arc<postmark::reqwest::PostmarkClient>>,
    req: actix_web::web::Json<crate::routes::auth::password_reset_request::PasswordResetRequest>,
) -> impl actix_web::Responder {
    match crate::user_management::initiate_password_reset(&pool, &postmark_client, &req.email).await {
        Ok(_) => actix_web::HttpResponse::Ok()
            .json("If your email is registered, a password reset link has been sent."),
        Err(e) => {
            log::error!("Password reset initiation failed for {}: {}", req.email, e);
            actix_web::HttpResponse::Ok()
                .json("If your email is registered, a password reset link has been sent.")
        }
    }
}