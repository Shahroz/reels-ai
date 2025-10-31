//! Cohort funnel analysis API endpoint for analytics dashboard.
//!
//! Provides REST API access to cohort-based funnel analysis functionality.
//! Handles user input validation and coordinates with service layer.
//! Returns detailed funnel metrics for selected user cohorts.
//! Supports admin-only access with comprehensive error handling.

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CohortFunnelAnalysisRequest {
    #[schema(example = "2025-01-01", format = "date")]
    pub registration_date_start: chrono::NaiveDate,
    #[schema(example = "2025-01-31", format = "date")]
    pub registration_date_end: chrono::NaiveDate,
    #[schema(example = 30)]
    pub analysis_period_days: Option<u32>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct CohortFunnelAnalysisResponse {
    pub total_cohort_users: i64,
    pub funnel_steps: Vec<FunnelStepResponse>,
    #[schema(example = "2025-01-01", format = "date")]
    pub cohort_period: chrono::NaiveDate,
    #[schema(value_type = String, format = "date-time", example = "2024-04-21T10:00:00Z")]
    pub analysis_generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct FunnelStepResponse {
    pub step_name: String,
    pub unique_users: i64,
    pub total_events: i64,
    pub conversion_rate_from_previous: Option<f64>,
    pub conversion_rate_from_start: f64,
}

#[utoipa::path(
    get,
    path = "/api/analytics/cohort-funnel-analysis",
    tag = "Analytics",
    summary = "Get cohort funnel analysis",
    description = "Performs comprehensive cohort-based funnel analysis by registration date",
    params(
        ("registration_date_start" = chrono::NaiveDate, Query, description = "Start date for user registration cohort"),
        ("registration_date_end" = chrono::NaiveDate, Query, description = "End date for user registration cohort"),
        ("analysis_period_days" = Option<u32>, Query, description = "Optional analysis period in days")
    ),
    responses(
        (status = 200, description = "Cohort funnel analysis data", body = CohortFunnelAnalysisResponse),
        (status = 400, description = "Invalid request parameters"),
        (status = 401, description = "Unauthorized - authentication required"),
        (status = 403, description = "Forbidden - admin access required"),
        (status = 500, description = "Internal server error")
    ),
    security(("jwt_auth" = []))
)]
#[actix_web::get("/cohort-funnel-analysis")]
pub async fn get_cohort_funnel_analysis(
    query: actix_web::web::Query<CohortFunnelAnalysisRequest>,
    db_pool: actix_web::web::Data<sqlx::PgPool>,
    user_claims: crate::auth::tokens::Claims,
) -> actix_web::Result<actix_web::web::Json<CohortFunnelAnalysisResponse>> {
    // Verify admin access
    if !user_claims.is_admin {
        return Err(actix_web::error::ErrorForbidden("Admin access required for analytics"));
    }

    // Create analytics service
    let service = crate::services::analytics::CohortFunnelService::new(db_pool.get_ref().clone());

    // Perform cohort funnel analysis
    let result = service
        .get_cohort_funnel_analysis(
            query.registration_date_start,
            query.registration_date_end,
            query.analysis_period_days,
        )
        .await;

    match result {
        Ok(analysis) => {
            let response = CohortFunnelAnalysisResponse {
                total_cohort_users: analysis.total_cohort_users,
                funnel_steps: analysis
                    .funnel_steps
                    .into_iter()
                    .map(|step| FunnelStepResponse {
                        step_name: step.step_name,
                        unique_users: step.unique_users,
                        total_events: step.total_events,
                        conversion_rate_from_previous: step.conversion_rate_from_previous,
                        conversion_rate_from_start: step.conversion_rate_from_start,
                    })
                    .collect(),
                cohort_period: analysis.cohort_period,
                analysis_generated_at: analysis.analysis_generated_at,
            };

            Ok(actix_web::web::Json(response))
        }
        Err(service_error) => {
            tracing::error!("Cohort funnel analysis error: {:?}", service_error);
            
            let error = match service_error {
                crate::services::analytics::CohortServiceError::InvalidDateRange => {
                    actix_web::error::ErrorBadRequest("End date must be after start date")
                }
                crate::services::analytics::CohortServiceError::DateRangeTooLarge => {
                    actix_web::error::ErrorBadRequest("Date range cannot exceed 365 days")
                }
                crate::services::analytics::CohortServiceError::FutureDateNotAllowed => {
                    actix_web::error::ErrorBadRequest("Registration dates cannot be in the future")
                }
                crate::services::analytics::CohortServiceError::DatabaseError(_) => {
                    actix_web::error::ErrorInternalServerError("An error occurred while processing the request")
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
    fn test_cohort_funnel_analysis_request_validation() {
        let request = super::CohortFunnelAnalysisRequest {
            registration_date_start: chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            registration_date_end: chrono::NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
            analysis_period_days: Some(30),
        };
        
        assert!(request.registration_date_end > request.registration_date_start);
        assert_eq!(request.analysis_period_days, Some(30));
    }

    #[test]
    fn test_funnel_step_response_creation() {
        let response = super::FunnelStepResponse {
            step_name: String::from("GET /dashboard"),
            unique_users: 150,
            total_events: 300,
            conversion_rate_from_previous: Some(75.0),
            conversion_rate_from_start: 75.0,
        };
        
        assert_eq!(response.step_name, "GET /dashboard");
        assert_eq!(response.unique_users, 150);
        assert!(response.conversion_rate_from_previous.is_some());
    }
} 