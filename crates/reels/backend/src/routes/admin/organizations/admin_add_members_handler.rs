//! Handler for batch adding members to an organization via admin endpoint.
//!
//! This endpoint allows administrators to add multiple users to an organization by email.
//! Returns 207 Multi-Status with detailed success/failure results for each email.
//! The handler validates the request and delegates to the service layer which handles
//! the complete business operation including transaction management and audit logging.

#[utoipa::path(
    post,
    path = "/api/admin/organizations/{organization_id}/members/batch",
    tag = "Admin",
    params(
        ("organization_id" = uuid::Uuid, Path, description = "Organization ID to add members to")
    ),
    request_body = crate::routes::admin::organizations::admin_add_members_request::AdminAddMembersRequest,
    responses(
        (status = 207, description = "Multi-Status - partial success", body = crate::routes::admin::organizations::admin_add_members_response::AdminAddMembersResponse),
        (status = 400, description = "Bad request - invalid input", body = crate::routes::error_response::ErrorResponse),
        (status = 401, description = "Unauthorized - user is not an admin", body = crate::routes::error_response::ErrorResponse),
        (status = 404, description = "Organization not found", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[actix_web::post("/{organization_id}/members/batch")]
#[tracing::instrument(skip(pool, postmark_client, auth_claims, payload))]
pub async fn admin_add_members_handler(
    pool: actix_web::web::Data<sqlx::PgPool>,
    postmark_client: actix_web::web::Data<std::sync::Arc<postmark::reqwest::PostmarkClient>>,
    auth_claims: crate::auth::tokens::Claims,
    organization_id: actix_web::web::Path<uuid::Uuid>,
    payload: actix_web::web::Json<
        crate::routes::admin::organizations::admin_add_members_request::AdminAddMembersRequest,
    >,
) -> impl actix_web::Responder {
    match crate::queries::admin::organizations::services::add_members_service(
        pool.get_ref(),
        postmark_client.as_ref(),
        auth_claims.user_id,
        organization_id.into_inner(),
        payload.emails.clone(),
        payload.role.clone(),
    )
    .await
    {
        Ok(result) => {
            let success_dtos: Vec<
                crate::routes::admin::organizations::admin_add_members_response::MemberAddSuccess,
            > = result
                .success
                .into_iter()
                .map(|s| {
                    crate::routes::admin::organizations::admin_add_members_response::MemberAddSuccess {
                        email: s.email,
                        user_id: s.user_id,
                        member: s.member,
                    }
                })
                .collect();

            let failed_dtos: Vec<
                crate::routes::admin::organizations::admin_add_members_response::MemberAddFailure,
            > = result
                .failed
                .into_iter()
                .map(|f| {
                    crate::routes::admin::organizations::admin_add_members_response::MemberAddFailure {
                        email: f.email,
                        reason: f.reason,
                    }
                })
                .collect();

            let response = crate::routes::admin::organizations::admin_add_members_response::AdminAddMembersResponse {
                success: success_dtos,
                failed: failed_dtos,
            };

            actix_web::HttpResponse::MultiStatus().json(response)
        }
        Err(e) => {
            // Use typed error methods to determine HTTP status code
            if e.is_not_found() {
                log::warn!(
                    "Admin {} tried to add members to non-existent organization: {}",
                    auth_claims.user_id,
                    e
                );
                actix_web::HttpResponse::NotFound().json(
                    crate::routes::error_response::ErrorResponse {
                        error: e.to_string(),
                    },
                )
            } else if e.is_client_error() {
                log::warn!(
                    "Admin {} provided invalid input for adding members: {}",
                    auth_claims.user_id,
                    e
                );
                actix_web::HttpResponse::BadRequest().json(
                    crate::routes::error_response::ErrorResponse {
                        error: e.to_string(),
                    },
                )
            } else {
                log::error!(
                    "Admin {} failed to add members: {}",
                    auth_claims.user_id,
                    e
                );
                actix_web::HttpResponse::InternalServerError().json(
                    crate::routes::error_response::ErrorResponse {
                        error: String::from("Failed to add members."),
                    },
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typed_error_not_found() {
        // Test that OrganizationNotFound is correctly identified as not found
        let error = crate::queries::admin::admin_service_error::AdminServiceError::OrganizationNotFound;
        assert!(error.is_not_found());
        assert!(!error.is_client_error());
    }

    #[test]
    fn test_typed_error_empty_email_list() {
        // Test that EmptyEmailList is correctly identified as client error
        let error = crate::queries::admin::admin_service_error::AdminServiceError::EmptyEmailList;
        assert!(error.is_client_error());
        assert!(!error.is_not_found());
    }

    #[test]
    fn test_typed_error_too_many_emails() {
        // Test that TooManyEmails is correctly identified as client error
        let error = crate::queries::admin::admin_service_error::AdminServiceError::TooManyEmails {
            max: 50,
            actual: 100,
        };
        assert!(error.is_client_error());
        assert!(!error.is_not_found());
        
        // Verify error message contains the values
        let error_msg = error.to_string();
        assert!(error_msg.contains("50"));
        assert!(error_msg.contains("100"));
    }

    #[test]
    fn test_response_dto_construction() {
        // Test that response DTOs can be constructed correctly
        let success = crate::routes::admin::organizations::admin_add_members_response::MemberAddSuccess {
            email: String::from("test@example.com"),
            user_id: uuid::Uuid::new_v4(),
            member: crate::db::organization_members::OrganizationMember {
                organization_id: uuid::Uuid::new_v4(),
                user_id: uuid::Uuid::new_v4(),
                role: String::from("member"),
                status: String::from("active"),
                invited_by_user_id: None,
                invited_at: None,
                joined_at: Some(chrono::Utc::now()),
            },
        };
        
        assert_eq!(success.email, "test@example.com");
    }

    #[test]
    fn test_failure_dto_construction() {
        // Test that failure DTOs can be constructed correctly
        let failure = crate::routes::admin::organizations::admin_add_members_response::MemberAddFailure {
            email: String::from("invalid@example.com"),
            reason: String::from("User already a member"),
        };
        
        assert_eq!(failure.email, "invalid@example.com");
        assert_eq!(failure.reason, "User already a member");
    }
}
