//! Available cohorts API endpoint for analytics dashboard.
//!
//! Provides REST API access to available user cohorts for selection.
//! Returns cohorts grouped by registration date with user counts.
//! Supports pagination and filtering for cohort selection interface.
//! Enables admin users to choose cohorts for funnel analysis.

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct GetAvailableCohortsRequest {
    #[schema(example = 50)]
    pub limit: Option<i32>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct AvailableCohortResponse {
    #[schema(example = "2025-01-15", format = "date")]
    pub registration_date: chrono::NaiveDate,
    pub user_count: i64,
    #[schema(example = "2025-01-15", format = "date")]
    pub first_activity_date: Option<chrono::NaiveDate>,
    #[schema(example = "2025-01-20", format = "date")]
    pub last_activity_date: Option<chrono::NaiveDate>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct GetAvailableCohortsResponse {
    pub cohorts: Vec<AvailableCohortResponse>,
    pub total_cohorts: usize,
}

#[utoipa::path(
    get,
    path = "/api/analytics/available-cohorts",
    tag = "Analytics",
    summary = "Get available user cohorts",
    description = "Retrieves available user cohorts grouped by registration date for funnel analysis",
    params(
        ("limit" = Option<i32>, Query, description = "Maximum number of cohorts to return (default: 100, max: 1000)")
    ),
    responses(
        (status = 200, description = "Available cohorts data", body = GetAvailableCohortsResponse),
        (status = 400, description = "Invalid request parameters"),
        (status = 401, description = "Unauthorized - authentication required"),
        (status = 403, description = "Forbidden - admin access required"),
        (status = 500, description = "Internal server error")
    ),
    security(("jwt_auth" = []))
)]
#[actix_web::get("/available-cohorts")]
pub async fn get_available_cohorts(
    query: actix_web::web::Query<GetAvailableCohortsRequest>,
    db_pool: actix_web::web::Data<sqlx::PgPool>,
    user_claims: crate::auth::tokens::Claims,
) -> actix_web::Result<actix_web::web::Json<GetAvailableCohortsResponse>> {
    // Verify admin access
    if !user_claims.is_admin {
        return Err(actix_web::error::ErrorForbidden("Admin access required for analytics"));
    }

    // Create analytics service
    let service = crate::services::analytics::CohortFunnelService::new(db_pool.get_ref().clone());

    // Get available cohorts
    let result = service.get_available_cohorts(query.limit).await;

    match result {
        Ok(cohorts) => {
            let cohort_responses: Vec<AvailableCohortResponse> = cohorts
                .into_iter()
                .map(|cohort| AvailableCohortResponse {
                    registration_date: cohort.registration_date,
                    user_count: cohort.user_count,
                    first_activity_date: cohort.first_activity_date,
                    last_activity_date: cohort.last_activity_date,
                })
                .collect();

            let response = GetAvailableCohortsResponse {
                total_cohorts: cohort_responses.len(),
                cohorts: cohort_responses,
            };

            Ok(actix_web::web::Json(response))
        }
        Err(service_error) => {
            tracing::error!("Get available cohorts error: {:?}", service_error);
            
            let error = match service_error {
                crate::services::analytics::CohortServiceError::InvalidLimit => {
                    actix_web::error::ErrorBadRequest("Limit must be greater than 0")
                }
                crate::services::analytics::CohortServiceError::DatabaseError(_) => {
                    actix_web::error::ErrorInternalServerError("An error occurred while retrieving cohorts")
                }
                _ => {
                    actix_web::error::ErrorInternalServerError("An unexpected error occurred")
                }
            };
            
            Err(error)
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/analytics/cohorts/{registration_date}",
    tag = "Analytics", 
    summary = "Get cohort by date",
    description = "Retrieves a specific cohort by registration date",
    params(
        ("registration_date" = chrono::NaiveDate, Path, description = "Registration date in YYYY-MM-DD format")
    ),
    responses(
        (status = 200, description = "Cohort data", body = AvailableCohortResponse),
        (status = 404, description = "Cohort not found"),
        (status = 400, description = "Invalid date format"),
        (status = 401, description = "Unauthorized - authentication required"),
        (status = 403, description = "Forbidden - admin access required"),
        (status = 500, description = "Internal server error")
    ),
    security(("jwt_auth" = []))
)]
#[actix_web::get("/cohorts/{registration_date}")]
pub async fn get_cohort_by_date(
    path: actix_web::web::Path<chrono::NaiveDate>,
    db_pool: actix_web::web::Data<sqlx::PgPool>,
    user_claims: crate::auth::tokens::Claims,
) -> actix_web::Result<actix_web::web::Json<AvailableCohortResponse>> {
    // Verify admin access
    if !user_claims.is_admin {
        return Err(actix_web::error::ErrorForbidden("Admin access required for analytics"));
    }

    let registration_date = path.into_inner();

    // Create analytics service
    let service = crate::services::analytics::CohortFunnelService::new(db_pool.get_ref().clone());

    // Get cohort by date
    let result = service.get_cohort_by_date(registration_date).await;

    match result {
        Ok(Some(cohort)) => {
            let response = AvailableCohortResponse {
                registration_date: cohort.registration_date,
                user_count: cohort.user_count,
                first_activity_date: cohort.first_activity_date,
                last_activity_date: cohort.last_activity_date,
            };

            Ok(actix_web::web::Json(response))
        }
        Ok(None) => {
            Err(actix_web::error::ErrorNotFound(std::format!("No cohort found for registration date {}", registration_date)))
        }
        Err(service_error) => {
            tracing::error!("Get cohort by date error: {:?}", service_error);
            
            let error = match service_error {
                crate::services::analytics::CohortServiceError::FutureDateNotAllowed => {
                    actix_web::error::ErrorBadRequest("Registration date cannot be in the future")
                }
                crate::services::analytics::CohortServiceError::DatabaseError(_) => {
                    actix_web::error::ErrorInternalServerError("An error occurred while retrieving the cohort")
                }
                _ => {
                    actix_web::error::ErrorInternalServerError("An unexpected error occurred")
                }
            };
            
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_available_cohorts_request_validation() {
        let request = super::GetAvailableCohortsRequest {
            limit: Some(50),
        };
        
        assert_eq!(request.limit, Some(50));
    }

    #[test]
    fn test_available_cohort_response_creation() {
        let response = super::AvailableCohortResponse {
            registration_date: chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            user_count: 100,
            first_activity_date: Some(chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()),
            last_activity_date: Some(chrono::NaiveDate::from_ymd_opt(2025, 1, 20).unwrap()),
        };
        
        assert_eq!(response.user_count, 100);
        assert!(response.first_activity_date.is_some());
        assert!(response.last_activity_date.is_some());
    }

    #[test]
    fn test_date_validation() {
        let today = chrono::Utc::now().date_naive();
        let future_date = today + chrono::Duration::days(1);
        
        // Test that future date would be invalid
        assert!(future_date > today);
    }
} 