//! Handler for providing Key Performance Indicator (KPI) summaries.
//!
//! Defines the GET `/api/dashboard/kpi-summary` endpoint.
//! This endpoint returns a summary of key metrics including total counts,
//! counts for various recent time periods (today, yesterday, this/last week, this/last month),
//! and percentage changes for user activity, content creation, etc.
//! Access to this endpoint is restricted to administrators.

//! Revision History
//! - 2025-05-20T13:07:38Z @AI: Initial implementation of KPI summary endpoint.

// No `use` statements at module level as per guidelines. Full paths are used.

use crate::queries::dashboard::query_kpi_metrics::{build_kpi_metric, KpiMetric};

/// Query parameters for the KPI summary endpoint (currently none).
#[derive(serde::Deserialize, Debug, utoipa::ToSchema, utoipa::IntoParams)]
pub struct KpiParams {
    // No parameters for now, but struct is here for future extensibility.
}

/// Response structure for KPI summary, containing metrics for various entities.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct KpiSummaryResponse {
    pub users: KpiMetric,
    pub documents: KpiMetric,
    pub creatives: KpiMetric,
    pub styles: KpiMetric,
    pub assets: KpiMetric,
    pub custom_creative_formats: KpiMetric,
}

/// Holds all calculated date ranges for KPI queries.
pub struct AllDateRanges {
    pub today_start: chrono::DateTime<chrono::Utc>,
    pub today_end: chrono::DateTime<chrono::Utc>,
    pub yesterday_start: chrono::DateTime<chrono::Utc>,
    pub yesterday_end: chrono::DateTime<chrono::Utc>,
    pub this_week_start: chrono::DateTime<chrono::Utc>,
    pub this_week_end: chrono::DateTime<chrono::Utc>,
    pub last_week_start: chrono::DateTime<chrono::Utc>,
    pub last_week_end: chrono::DateTime<chrono::Utc>,
    pub this_month_start: chrono::DateTime<chrono::Utc>,
    pub this_month_end: chrono::DateTime<chrono::Utc>,
    pub last_month_start: chrono::DateTime<chrono::Utc>,
    pub last_month_end: chrono::DateTime<chrono::Utc>,
    pub current_7_days_start: chrono::DateTime<chrono::Utc>,
    pub current_7_days_end: chrono::DateTime<chrono::Utc>,
    pub previous_7_days_start: chrono::DateTime<chrono::Utc>,
    pub previous_7_days_end: chrono::DateTime<chrono::Utc>,
    pub current_30_days_start: chrono::DateTime<chrono::Utc>,
    pub current_30_days_end: chrono::DateTime<chrono::Utc>,
    pub previous_30_days_start: chrono::DateTime<chrono::Utc>,
    pub previous_30_days_end: chrono::DateTime<chrono::Utc>,
}

/// Calculates all necessary date ranges based on the current time.
/// End dates are exclusive for queries (e.g., `created_at < end_date`).
fn calculate_all_date_ranges(now: chrono::DateTime<chrono::Utc>) -> AllDateRanges {
    use chrono::Datelike; // Allowed for methods on chrono types

    let today_naive_date = now.date_naive();
    let today_start = today_naive_date.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(chrono::Utc).unwrap();
    let today_end = today_start + chrono::Duration::days(1);

    let yesterday_start = today_start - chrono::Duration::days(1);
    let yesterday_end = today_start; // Exclusive end, so it's the start of today

    let days_from_monday = today_naive_date.weekday().num_days_from_monday() as i64;
    let this_week_start = today_start - chrono::Duration::days(days_from_monday);
    let this_week_end = this_week_start + chrono::Duration::weeks(1);

    let last_week_start = this_week_start - chrono::Duration::weeks(1);
    let last_week_end = this_week_start;

    let this_month_start = chrono::NaiveDate::from_ymd_opt(today_naive_date.year(), today_naive_date.month(), 1).unwrap().and_hms_opt(0, 0, 0).unwrap().and_local_timezone(chrono::Utc).unwrap();
    let next_month_naive = if today_naive_date.month() == 12 {
        chrono::NaiveDate::from_ymd_opt(today_naive_date.year() + 1, 1, 1).unwrap()
    } else {
        chrono::NaiveDate::from_ymd_opt(today_naive_date.year(), today_naive_date.month() + 1, 1).unwrap()
    };
    let this_month_end = next_month_naive.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(chrono::Utc).unwrap();

    let last_month_end = this_month_start;
    let last_month_naive_start = if today_naive_date.month() == 1 {
        chrono::NaiveDate::from_ymd_opt(today_naive_date.year() - 1, 12, 1).unwrap()
    } else {
        chrono::NaiveDate::from_ymd_opt(today_naive_date.year(), today_naive_date.month() - 1, 1).unwrap()
    };
    let last_month_start = last_month_naive_start.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(chrono::Utc).unwrap();

    let current_7_days_end = yesterday_end; // Ends at the start of today (i.e., includes all of yesterday)
    let current_7_days_start = current_7_days_end - chrono::Duration::days(7);
    let previous_7_days_end = current_7_days_start;
    let previous_7_days_start = previous_7_days_end - chrono::Duration::days(7);

    let current_30_days_end = yesterday_end;
    let current_30_days_start = current_30_days_end - chrono::Duration::days(30);
    let previous_30_days_end = current_30_days_start;
    let previous_30_days_start = previous_30_days_end - chrono::Duration::days(30);

    AllDateRanges {
        today_start, today_end,
        yesterday_start, yesterday_end,
        this_week_start, this_week_end,
        last_week_start, last_week_end,
        this_month_start, this_month_end,
        last_month_start, last_month_end,
        current_7_days_start, current_7_days_end,
        previous_7_days_start, previous_7_days_end,
        current_30_days_start, current_30_days_end,
        previous_30_days_start, previous_30_days_end,
    }
}

#[utoipa::path(
    get,
    path = "/api/dashboard/kpi-summary",
    tag = "Dashboard",
    params(KpiParams),
    responses(
        (status = 200, description = "KPI summary retrieved successfully.", body = KpiSummaryResponse),
        (status = 403, description = "Forbidden. Administrator access required.", body = crate::routes::error_response::ErrorResponse),
        (status = 500, description = "Internal Server Error. Failed to retrieve KPI data.", body = crate::routes::error_response::ErrorResponse)
    ),
    security(
        ("user_auth" = [])
    )
)]
#[tracing::instrument(name = "get_kpi_summary", skip(pool, claims, _params), fields(user_id = %claims.user_id, is_admin = %claims.is_admin))]
#[actix_web::get("/kpi-summary")]
pub async fn get_kpi_summary(
    pool: actix_web::web::Data<sqlx::PgPool>,
    claims: actix_web::web::ReqData<crate::auth::tokens::Claims>,
    _params: actix_web::web::Query<KpiParams>, // Not used for now
) -> impl actix_web::Responder {
    if !claims.is_admin {
        return actix_web::HttpResponse::Forbidden().json(
            crate::routes::error_response::ErrorResponse {
                error: std::string::String::from("Administrator access required."),
            },
        );
    }

    let now = chrono::Utc::now();
    let dates = calculate_all_date_ranges(now);
    let db_pool_ref = pool.get_ref();

    let users_fut = build_kpi_metric(db_pool_ref, "users", "Users", &dates);
    let documents_fut = build_kpi_metric(db_pool_ref, "documents", "Documents", &dates);
    let creatives_fut = build_kpi_metric(db_pool_ref, "creatives", "Creatives", &dates);
    let styles_fut = build_kpi_metric(db_pool_ref, "styles", "Styles", &dates);
    let assets_fut = build_kpi_metric(db_pool_ref, "assets", "Assets", &dates);
   let custom_formats_fut = build_kpi_metric(db_pool_ref, "custom_creative_formats", "Custom Creative Formats", &dates);

   match futures::try_join!(
       users_fut,
       documents_fut,
       creatives_fut,
       styles_fut,
      assets_fut,
      custom_formats_fut
  ) {
      Ok((users, documents, creatives, styles, assets, custom_creative_formats)) => {
          let response = KpiSummaryResponse {
              users,
                documents,
                creatives,
                styles,
                assets,
                custom_creative_formats,
            };
            actix_web::HttpResponse::Ok().json(response)
        }
        Err(e) => {
            log::error!("Failed to build one or more KPI metrics: {e:?}");
            actix_web::HttpResponse::InternalServerError().json(
                crate::routes::error_response::ErrorResponse {
                    error: std::string::String::from("Failed to retrieve KPI data due to a database error."),
                },
            )
        }
    }
}
