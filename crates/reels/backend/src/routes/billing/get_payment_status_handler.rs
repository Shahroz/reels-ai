//! HTTP handler for retrieving recent payment completion status.
//!
//! This handler processes GET requests to /api/billing/payment-status and returns
//! information about recently completed payments for the authenticated user.
//! Used to support frontend payment completion detection workflows and
//! provide real-time feedback about payment processing status.
//! 
//! Revision History:
//! - 2025-09-17T20:45:00Z @AI: Created during organization-based billing implementation
//! - [Prior updates not documented in original file]

#[utoipa::path(
    get,
    path = "/api/billing/payment-status",
    tag = "Billing",
    responses(
        (status = 200, description = "Payment status retrieved successfully", body = crate::routes::billing::payment_status_response::PaymentStatusResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
#[actix_web::get("/payment-status")]
#[tracing::instrument(skip(pool, req))]
pub async fn get_payment_status_handler(
    pool: actix_web::web::Data<sqlx::PgPool>,
    req: actix_web::HttpRequest,
) -> impl actix_web::Responder {
    let user_id = extract_user_id_from_request(&req);
    let user_id = match user_id {
        std::result::Result::Ok(id) => id,
        std::result::Result::Err(response) => return response,
    };

    match crate::db::payment_completions::get_recent_payment_completion(&pool, user_id, 5).await {
        std::result::Result::Ok(payment_completion) => {
            let response = crate::routes::billing::payment_status_response::PaymentStatusResponse {
                is_completed: payment_completion.is_some(),
                payment_method: payment_completion.as_ref().map(|pc| pc.payment_method.clone()),
                completed_at: payment_completion.as_ref().map(|pc| pc.completed_at),
                session_id: payment_completion.as_ref().map(|pc| pc.session_id.clone()),
            };

            actix_web::HttpResponse::Ok().json(response)
        }
        std::result::Result::Err(e) => {
            log::error!("Failed to get payment status for user {user_id}: {e}");
            actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error",
                "message": "Failed to retrieve payment status"
            }))
        }
    }
}

fn extract_user_id_from_request(req: &actix_web::HttpRequest) -> std::result::Result<uuid::Uuid, actix_web::HttpResponse> {
    if let std::option::Option::Some(auth_user) = actix_web::HttpMessage::extensions(req).get::<crate::middleware::auth::AuthenticatedUser>() {
        let user_id = match auth_user {
            crate::middleware::auth::AuthenticatedUser::Jwt(claims) => claims.user_id,
            crate::middleware::auth::AuthenticatedUser::ApiKey(user_id) => *user_id,
        };
        std::result::Result::Ok(user_id)
    } else {
        std::result::Result::Err(actix_web::HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Unauthorized",
            "message": "Authentication required"
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_user_id_from_jwt_request() {
        let mut req = actix_web::test::TestRequest::default().to_http_request();
        let user_id = uuid::Uuid::new_v4();
        let claims = crate::auth::tokens::claims::Claims {
            user_id,
            is_admin: false,
            email: "test@example.com".to_string(),
            email_verified: true,
            feature_flags: std::option::Option::Some(std::vec::Vec::new()),
            exp: 1234567890,
            admin_id: std::option::Option::None,
            is_impersonating: std::option::Option::Some(false),
        };
        actix_web::HttpMessage::extensions_mut(&mut req).insert(crate::middleware::auth::AuthenticatedUser::Jwt(claims));
        
        let result = extract_user_id_from_request(&req);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), user_id);
    }

    #[test]
    fn test_extract_user_id_from_api_key_request() {
        let mut req = actix_web::test::TestRequest::default().to_http_request();
        let user_id = uuid::Uuid::new_v4();
        actix_web::HttpMessage::extensions_mut(&mut req).insert(crate::middleware::auth::AuthenticatedUser::ApiKey(user_id));
        
        let result = extract_user_id_from_request(&req);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), user_id);
    }

    #[test]
    fn test_extract_user_id_unauthorized_request() {
        let req = actix_web::test::TestRequest::default().to_http_request();
        let result = extract_user_id_from_request(&req);
        assert!(result.is_err());
    }
}
