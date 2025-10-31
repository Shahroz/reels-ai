//! Handler for comprehensive user cohort analysis endpoint.
//!
//! Defines the GET `/api/dashboard/user-cohort-analysis` endpoint.
//! Provides detailed analysis of user activity patterns within selected cohorts,
//! supporting both registration-based and login-based cohort selection.
//! Returns comprehensive metrics for asset activities and other user engagements.
//!
//! Revision History
//! - 2025-10-14T00:00:00Z @AI: Initial implementation of comprehensive cohort analysis endpoint.

use crate::routes::error_response::ErrorResponse;
use crate::queries::dashboard::query_user_cohort_analysis::{
    get_user_cohort_analysis, UserCohortAnalysisParams as QueryParams, 
    CohortSelectionMethod
};

/// Parameters for requesting comprehensive user cohort analysis
#[derive(serde::Deserialize, Debug, utoipa::ToSchema, utoipa::IntoParams)]
pub struct UserCohortAnalysisParams {
    /// Method for selecting cohort users: "registration" or "login"
    #[schema(example = "registration")]
    pub cohort_selection_method: String,
    /// Start date for cohort selection (YYYY-MM-DD)
    #[schema(value_type = String, format = "date", example = "2023-01-01")]
    pub cohort_start_date: chrono::NaiveDate,
    /// End date for cohort selection (YYYY-MM-DD)
    #[schema(value_type = String, format = "date", example = "2023-01-31")]
    pub cohort_end_date: chrono::NaiveDate,
    /// Start date for activity analysis (YYYY-MM-DD)
    #[schema(value_type = String, format = "date", example = "2023-01-01")]
    pub analysis_start_date: chrono::NaiveDate,
    /// End date for activity analysis (YYYY-MM-DD)
    #[schema(value_type = String, format = "date", example = "2023-12-31")]
    pub analysis_end_date: chrono::NaiveDate,
    /// Start date for user metrics time series (YYYY-MM-DD)
    #[schema(value_type = String, format = "date", example = "2023-01-01")]
    pub user_metrics_start_date: chrono::NaiveDate,
    /// End date for user metrics time series (YYYY-MM-DD)
    #[schema(value_type = String, format = "date", example = "2023-12-31")]
    pub user_metrics_end_date: chrono::NaiveDate,
}

/// Cohort summary statistics
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct CohortSummary {
    pub total_users: i64,
    pub paying_users: i64,
    pub paying_users_percentage: f64,
}

/// Daily activity data point
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct DailyActivityPoint {
    #[schema(value_type = String, format = "date")]
    pub date: chrono::NaiveDate,
    pub count: i64,
}

/// Baseline count for cumulative metrics (count before analysis start date)
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct BaselineCounts {
    pub user_registrations: i64,
    pub paying_users: i64,
    pub total_uploads: i64,
    pub manual_uploads: i64,
    pub walkthrough_uploads: i64,
    pub total_enhancements: i64,
    pub manual_upload_enhancements: i64,
    pub walkthrough_upload_enhancements: i64,
    pub total_asset_creation: i64,
    pub walkthrough_creation: i64,
    pub document_uploads: i64,
    pub listing_creation: i64,
    pub marketing_creative_usage: i64,
}

/// Activity metrics for a specific activity type (with time series)
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ActivityMetrics {
    pub users_count: i64,
    pub users_percentage: f64,
    pub total_events: i64,
    pub avg_events_per_user: f64,
    pub daily_series: Vec<DailyActivityPoint>,
}

/// Asset-related activities breakdown
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct AssetActivities {
    pub manual_uploads: ActivityMetrics,
    pub walkthrough_uploads: ActivityMetrics,
    pub total_uploads: ActivityMetrics,
    pub manual_upload_enhancements: ActivityMetrics,
    pub walkthrough_upload_enhancements: ActivityMetrics,
    pub total_enhancements: ActivityMetrics,
    pub total_asset_creation: ActivityMetrics,
}

/// Other user activities breakdown
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct OtherActivities {
    pub walkthrough_creation: ActivityMetrics,
    pub document_uploads: ActivityMetrics,
    pub listing_creation: ActivityMetrics,
    pub marketing_creative_usage: ActivityMetrics, // Renamed from marketing_asset_creation
}

/// Response structure for comprehensive user cohort analysis
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct UserCohortAnalysisResponse {
    pub cohort_period: String,
    pub analysis_period: String,
    pub cohort_selection_method: String,
    pub cohort_summary: CohortSummary,
    pub asset_activities: AssetActivities,    // Summary for cohort users
    pub other_activities: OtherActivities,    // Summary for cohort users
    pub generated_at: chrono::DateTime<chrono::Utc>,
    // User metrics / Time series (always all users)
    pub user_metrics_period: String,
    pub baseline_counts: BaselineCounts,      // Counts before user_metrics_start_date
    pub user_registration_series: Vec<DailyActivityPoint>,
    pub user_login_series: Vec<DailyActivityPoint>,
    pub paying_users_series: Vec<DailyActivityPoint>,
    pub asset_activities_series: AssetActivities,  // Time series for all users
    pub other_activities_series: OtherActivities,  // Time series for all users
}

#[utoipa::path(
    get,
    path = "/api/dashboard/user-cohort-analysis",
    tag = "Dashboard",
    params(UserCohortAnalysisParams),
    responses(
        (status = 200, description = "User cohort analysis retrieved successfully", body = UserCohortAnalysisResponse),
        (status = 400, description = "Invalid parameters", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse), 
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse),
        (status = 500, description = "Internal Server Error", body = ErrorResponse)
    ),
    security(("user_auth" = []))
)]
#[tracing::instrument(
    name = "get_user_cohort_analysis_handler",
    skip(pool, claims),
    fields(
        user_id = %claims.user_id,
        is_admin = %claims.is_admin,
        cohort_method = %params.cohort_selection_method,
        cohort_period = %format!("{} to {}", params.cohort_start_date, params.cohort_end_date),
        analysis_period = %format!("{} to {}", params.analysis_start_date, params.analysis_end_date)
    )
)]
#[actix_web::get("/user-cohort-analysis")]
pub async fn get_user_cohort_analysis_handler(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
    params: actix_web::web::Query<UserCohortAnalysisParams>,
) -> impl actix_web::Responder {
    // Admin access validation
    if !claims.is_admin {
        log::warn!(
            "Non-admin user {} attempted to access cohort analysis",
            claims.user_id
        );
        return actix_web::HttpResponse::Forbidden().json(ErrorResponse {
            error: "Admin access required.".to_string(),
        });
    }

    // Validate date ranges
    if params.cohort_start_date > params.cohort_end_date {
        return actix_web::HttpResponse::BadRequest().json(ErrorResponse {
            error: "Cohort start date cannot be after end date.".to_string(),
        });
    }

    if params.analysis_start_date > params.analysis_end_date {
        return actix_web::HttpResponse::BadRequest().json(ErrorResponse {
            error: "Analysis start date cannot be after end date.".to_string(),
        });
    }

    if params.user_metrics_start_date > params.user_metrics_end_date {
        return actix_web::HttpResponse::BadRequest().json(ErrorResponse {
            error: "User metrics start date cannot be after end date.".to_string(),
        });
    }

    // Parse cohort selection method
    let cohort_method = match params.cohort_selection_method.as_str() {
        "registration" => CohortSelectionMethod::ByRegistration,
        "login" => CohortSelectionMethod::ByLogin,
        _ => {
            return actix_web::HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid cohort_selection_method. Must be 'registration' or 'login'.".to_string(),
            });
        }
    };

    // Convert dates to DateTime for analysis period
    let analysis_start = params.analysis_start_date
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    let analysis_end = params.analysis_end_date
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .and_utc();

    // Convert dates to DateTime for user metrics period
    let user_metrics_start = params.user_metrics_start_date
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    let user_metrics_end = params.user_metrics_end_date
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .and_utc();

    // Execute comprehensive analysis
    let query_params = QueryParams {
        cohort_selection_method: cohort_method,
        cohort_start_date: params.cohort_start_date,
        cohort_end_date: params.cohort_end_date,
        analysis_start_date: analysis_start,
        analysis_end_date: analysis_end,
        user_metrics_start_date: user_metrics_start,
        user_metrics_end_date: user_metrics_end,
    };

    log::info!(
        "Executing cohort analysis for method: {}, cohort: {} to {}, analysis: {} to {}",
        params.cohort_selection_method,
        params.cohort_start_date,
        params.cohort_end_date,
        params.analysis_start_date,
        params.analysis_end_date
    );

    match get_user_cohort_analysis(pool.get_ref(), query_params).await {
        Ok(result) => {
            log::info!(
                "Cohort analysis completed: {} total users, {} paying users",
                result.cohort_summary.total_users,
                result.cohort_summary.paying_users
            );

            let response = UserCohortAnalysisResponse {
                cohort_period: format!("{} to {}", params.cohort_start_date, params.cohort_end_date),
                analysis_period: format!("{} to {}", params.analysis_start_date, params.analysis_end_date),
                cohort_selection_method: params.cohort_selection_method.clone(),
                cohort_summary: CohortSummary {
                    total_users: result.cohort_summary.total_users,
                    paying_users: result.cohort_summary.paying_users,
                    paying_users_percentage: result.cohort_summary.paying_users_percentage,
                },
                asset_activities: AssetActivities {
                    manual_uploads: convert_metrics_with_series(&result.asset_activities.manual_uploads),
                    walkthrough_uploads: convert_metrics_with_series(&result.asset_activities.walkthrough_uploads),
                    total_uploads: convert_metrics_with_series(&result.asset_activities.total_uploads),
                    manual_upload_enhancements: convert_metrics_with_series(&result.asset_activities.manual_upload_enhancements),
                    walkthrough_upload_enhancements: convert_metrics_with_series(&result.asset_activities.walkthrough_upload_enhancements),
                    total_enhancements: convert_metrics_with_series(&result.asset_activities.total_enhancements),
                    total_asset_creation: convert_metrics_with_series(&result.asset_activities.total_asset_creation),
                },
                other_activities: OtherActivities {
                    walkthrough_creation: convert_metrics_with_series(&result.other_activities.walkthrough_creation),
                    document_uploads: convert_metrics_with_series(&result.other_activities.document_uploads),
                    listing_creation: convert_metrics_with_series(&result.other_activities.listing_creation),
                    marketing_creative_usage: convert_metrics_with_series(&result.other_activities.marketing_creative_usage),
                },
                generated_at: chrono::Utc::now(),
                // User metrics / Time series (always all users)
                user_metrics_period: result.user_metrics_period,
                baseline_counts: BaselineCounts {
                    user_registrations: result.baseline_counts.user_registrations,
                    paying_users: result.baseline_counts.paying_users,
                    total_uploads: result.baseline_counts.total_uploads,
                    manual_uploads: result.baseline_counts.manual_uploads,
                    walkthrough_uploads: result.baseline_counts.walkthrough_uploads,
                    total_enhancements: result.baseline_counts.total_enhancements,
                    manual_upload_enhancements: result.baseline_counts.manual_upload_enhancements,
                    walkthrough_upload_enhancements: result.baseline_counts.walkthrough_upload_enhancements,
                    total_asset_creation: result.baseline_counts.total_asset_creation,
                    walkthrough_creation: result.baseline_counts.walkthrough_creation,
                    document_uploads: result.baseline_counts.document_uploads,
                    listing_creation: result.baseline_counts.listing_creation,
                    marketing_creative_usage: result.baseline_counts.marketing_creative_usage,
                },
                user_registration_series: result.user_registration_series.into_iter().map(|p| DailyActivityPoint {
                    date: p.date,
                    count: p.count,
                }).collect(),
                user_login_series: result.user_login_series.into_iter().map(|p| DailyActivityPoint {
                    date: p.date,
                    count: p.count,
                }).collect(),
                paying_users_series: result.paying_users_series.into_iter().map(|p| DailyActivityPoint {
                    date: p.date,
                    count: p.count,
                }).collect(),
                asset_activities_series: AssetActivities {
                    manual_uploads: convert_metrics_with_series(&result.asset_activities_series.manual_uploads),
                    walkthrough_uploads: convert_metrics_with_series(&result.asset_activities_series.walkthrough_uploads),
                    total_uploads: convert_metrics_with_series(&result.asset_activities_series.total_uploads),
                    manual_upload_enhancements: convert_metrics_with_series(&result.asset_activities_series.manual_upload_enhancements),
                    walkthrough_upload_enhancements: convert_metrics_with_series(&result.asset_activities_series.walkthrough_upload_enhancements),
                    total_enhancements: convert_metrics_with_series(&result.asset_activities_series.total_enhancements),
                    total_asset_creation: convert_metrics_with_series(&result.asset_activities_series.total_asset_creation),
                },
                other_activities_series: OtherActivities {
                    walkthrough_creation: convert_metrics_with_series(&result.other_activities_series.walkthrough_creation),
                    document_uploads: convert_metrics_with_series(&result.other_activities_series.document_uploads),
                    listing_creation: convert_metrics_with_series(&result.other_activities_series.listing_creation),
                    marketing_creative_usage: convert_metrics_with_series(&result.other_activities_series.marketing_creative_usage),
                },
            };

            actix_web::HttpResponse::Ok().json(response)
        }
        Err(e) => {
            log::error!("Failed to fetch user cohort analysis: {:?}", e);
            actix_web::HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to retrieve user cohort analysis.".to_string(),
            })
        }
    }
}

// Helper function to convert metrics with time series
fn convert_metrics_with_series(m: &crate::queries::dashboard::query_user_cohort_analysis::ActivityMetricsWithTimeSeries) -> ActivityMetrics {
    ActivityMetrics {
        users_count: m.users_count,
        users_percentage: m.users_percentage,
        total_events: m.total_events,
        avg_events_per_user: m.avg_events_per_user,
        daily_series: m.daily_series.iter().map(|p| DailyActivityPoint {
            date: p.date,
            count: p.count,
        }).collect(),
    }
}
