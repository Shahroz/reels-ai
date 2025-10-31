//! Comprehensive user cohort analysis queries for dashboard analytics.
//!
//! This module provides detailed analysis of user activity patterns within selected cohorts.
//! Supports cohort selection by registration date or login events, with separate analysis periods.
//! Tracks asset activities with source lineage and comprehensive user engagement metrics.
//!
//! Revision History
//! - 2025-10-14T00:00:00Z @AI: Initial implementation of comprehensive cohort analysis.

use sqlx::Row;

#[derive(Debug, Clone)]
pub enum CohortSelectionMethod {
    ByRegistration,  // Use users.created_at
    ByLogin,         // Use user_login_successful events
}

#[derive(Debug)]
pub struct UserCohortAnalysisParams {
    pub cohort_selection_method: CohortSelectionMethod,
    pub cohort_start_date: chrono::NaiveDate,
    pub cohort_end_date: chrono::NaiveDate,
    pub analysis_start_date: chrono::DateTime<chrono::Utc>,
    pub analysis_end_date: chrono::DateTime<chrono::Utc>,
    pub user_metrics_start_date: chrono::DateTime<chrono::Utc>,
    pub user_metrics_end_date: chrono::DateTime<chrono::Utc>,
}

/// Baseline counts before the user metrics start date
#[derive(Debug)]
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

pub struct UserCohortAnalysisResult {
    pub cohort_summary: CohortSummary,
    pub asset_activities: AssetActivitiesWithTimeSeries,   // Summary for cohort
    pub other_activities: OtherActivitiesWithTimeSeries,   // Summary for cohort
    // User metrics / Time series (always all users)
    pub user_metrics_period: String,
    pub baseline_counts: BaselineCounts,                    // Counts before user_metrics_start_date
    pub user_registration_series: Vec<DailyActivityPoint>,
    pub user_login_series: Vec<DailyActivityPoint>,
    pub paying_users_series: Vec<DailyActivityPoint>,
    pub asset_activities_series: AssetActivitiesWithTimeSeries,  // Time series for all users
    pub other_activities_series: OtherActivitiesWithTimeSeries,  // Time series for all users
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct CohortSummary {
    pub total_users: i64,
    pub paying_users: i64,
    pub paying_users_percentage: f64,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct AssetActivities {
    // A. Asset Upload Activities
    pub manual_uploads: ActivityMetrics,           // A1
    pub walkthrough_uploads: ActivityMetrics,      // A2  
    pub total_uploads: ActivityMetrics,            // A Total
    
    // B. Asset Enhancement Activities
    pub manual_upload_enhancements: ActivityMetrics,     // B1
    pub walkthrough_upload_enhancements: ActivityMetrics, // B2
    pub total_enhancements: ActivityMetrics,              // B Total
    
    // C. All Asset Creation
    pub total_asset_creation: ActivityMetrics,     // C
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct OtherActivities {
    pub walkthrough_creation: ActivityMetrics,     // D
    pub document_uploads: ActivityMetrics,         // E
    pub listing_creation: ActivityMetrics,         // F  
    pub marketing_asset_creation: ActivityMetrics, // G
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct ActivityMetrics {
    pub users_count: i64,
    pub users_percentage: f64,
    pub total_events: i64,
    pub avg_events_per_user: f64,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct DailyActivityPoint {
    pub date: chrono::NaiveDate,
    pub count: i64,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub struct ActivityMetricsWithTimeSeries {
    // Summary metrics (existing)
    pub users_count: i64,
    pub users_percentage: f64,
    pub total_events: i64,
    pub avg_events_per_user: f64,
    // New time series data
    pub daily_series: Vec<DailyActivityPoint>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct AssetActivitiesWithTimeSeries {
    // A. Asset Upload Activities
    pub manual_uploads: ActivityMetricsWithTimeSeries,
    pub walkthrough_uploads: ActivityMetricsWithTimeSeries,
    pub total_uploads: ActivityMetricsWithTimeSeries,
    
    // B. Asset Enhancement Activities
    pub manual_upload_enhancements: ActivityMetricsWithTimeSeries,
    pub walkthrough_upload_enhancements: ActivityMetricsWithTimeSeries,
    pub total_enhancements: ActivityMetricsWithTimeSeries,
    
    // C. All Asset Creation
    pub total_asset_creation: ActivityMetricsWithTimeSeries,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct OtherActivitiesWithTimeSeries {
    pub walkthrough_creation: ActivityMetricsWithTimeSeries,
    pub document_uploads: ActivityMetricsWithTimeSeries,
    pub listing_creation: ActivityMetricsWithTimeSeries,
    pub marketing_creative_usage: ActivityMetricsWithTimeSeries, // Renamed
}

/// Main entry point for comprehensive user cohort analysis
#[tracing::instrument(
    name = "get_user_cohort_analysis",
    skip(pool),
    fields(
        cohort_method = ?params.cohort_selection_method,
        cohort_start = %params.cohort_start_date,
        cohort_end = %params.cohort_end_date,
        analysis_start = %params.analysis_start_date,
        analysis_end = %params.analysis_end_date
    )
)]
pub async fn get_user_cohort_analysis(
    pool: &sqlx::PgPool,
    params: UserCohortAnalysisParams,
) -> Result<UserCohortAnalysisResult, sqlx::Error> {
    // Step 1: Get cohort users based on selection method
    let cohort_user_ids = get_cohort_users(pool, &params).await?;
    
    // Step 2: Get cohort summary (total and paying users) - uses cohort_user_ids and analysis dates
    let cohort_summary = get_cohort_summary(pool, &cohort_user_ids).await?;
    
    // Step 3: Get asset/activity SUMMARY for cohort
    let asset_activities = get_asset_activities_with_time_series(pool, &cohort_user_ids, &params).await?;
    let other_activities = get_other_activities_with_time_series(pool, &cohort_user_ids, &params).await?;
    
    // Step 4: Get TIME SERIES - ALWAYS ALL USERS (empty vec = no filter)
    // Create params with user_metrics dates for time series queries
    let user_metrics_params = UserCohortAnalysisParams {
        cohort_selection_method: params.cohort_selection_method.clone(),
        cohort_start_date: params.cohort_start_date,
        cohort_end_date: params.cohort_end_date,
        analysis_start_date: params.user_metrics_start_date,
        analysis_end_date: params.user_metrics_end_date,
        user_metrics_start_date: params.user_metrics_start_date,
        user_metrics_end_date: params.user_metrics_end_date,
    };
    
    let user_registration_series = get_user_registration_series(
        pool, 
        &Vec::new(),  // Empty = all users
        &user_metrics_params
    ).await?;
    
    let user_login_series = get_user_login_series(
        pool, 
        &Vec::new(),  // Empty = all users
        &user_metrics_params
    ).await?;
    
    let paying_users_series = get_paying_users_series(
        pool, 
        &Vec::new(),  // Empty = all users
        &user_metrics_params
    ).await?;
    
    let asset_activities_series = get_asset_activities_with_time_series(
        pool, 
        &Vec::new(),  // Empty = all users
        &user_metrics_params
    ).await?;
    
    let other_activities_series = get_other_activities_with_time_series(
        pool, 
        &Vec::new(),  // Empty = all users
        &user_metrics_params
    ).await?;
    
    // Format user metrics period string
    let user_metrics_period = format!(
        "{} to {}",
        user_metrics_params.analysis_start_date.format("%Y-%m-%d"),
        user_metrics_params.analysis_end_date.format("%Y-%m-%d")
    );
    
    // Step 5: Get baseline counts (counts before user_metrics_start_date)
    let baseline_counts = get_baseline_counts(pool, &params).await?;
    
    Ok(UserCohortAnalysisResult {
        cohort_summary,
        asset_activities,         // Summary for cohort
        other_activities,         // Summary for cohort
        user_metrics_period,
        baseline_counts,
        user_registration_series,
        user_login_series,
        paying_users_series,
        asset_activities_series,  // Time series for all users
        other_activities_series,  // Time series for all users
    })
}

/// Get user IDs based on cohort selection method and date range
async fn get_cohort_users(
    pool: &sqlx::PgPool,
    params: &UserCohortAnalysisParams,
) -> Result<Vec<uuid::Uuid>, sqlx::Error> {
    match params.cohort_selection_method {
        CohortSelectionMethod::ByRegistration => {
            let cohort_start = params.cohort_start_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
            let cohort_end = params.cohort_end_date.and_hms_opt(23, 59, 59).unwrap().and_utc();
            
            let rows = sqlx::query!(
                r#"
                SELECT id
                FROM users
                WHERE created_at >= $1 AND created_at <= $2
                "#,
                cohort_start,
                cohort_end
            )
            .fetch_all(pool)
            .await?;
            
            Ok(rows.into_iter().map(|row| row.id).collect())
        }
        CohortSelectionMethod::ByLogin => {
            let cohort_start = params.cohort_start_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
            let cohort_end = params.cohort_end_date.and_hms_opt(23, 59, 59).unwrap().and_utc();
            
            let rows = sqlx::query!(
                r#"
                SELECT DISTINCT user_id
                FROM analytics_events
                WHERE event_name = 'user_login_successful'
                    AND timestamp >= $1 AND timestamp <= $2
                    AND user_id IS NOT NULL
                "#,
                cohort_start,
                cohort_end
            )
            .fetch_all(pool)
            .await?;
            
            Ok(rows.into_iter().filter_map(|row| row.user_id).collect())
        }
    }
}

/// Get cohort summary with total and paying users
async fn get_cohort_summary(
    pool: &sqlx::PgPool,
    cohort_user_ids: &[uuid::Uuid],
) -> Result<CohortSummary, sqlx::Error> {
    if cohort_user_ids.is_empty() {
        return Ok(CohortSummary {
            total_users: 0,
            paying_users: 0,
            paying_users_percentage: 0.0,
        });
    }
    
    let total_users = cohort_user_ids.len() as i64;
    
    let paying_users_row = sqlx::query!(
        r#"
        SELECT COUNT(*) as paying_count
        FROM users
        WHERE id = ANY($1)
            AND subscription_status = 'active'
        "#,
        cohort_user_ids
    )
    .fetch_one(pool)
    .await?;
    
    let paying_users = paying_users_row.paying_count.unwrap_or(0);
    let paying_users_percentage = if total_users > 0 {
        (paying_users as f64 / total_users as f64) * 100.0
    } else {
        0.0
    };
    
    Ok(CohortSummary {
        total_users,
        paying_users,
        paying_users_percentage,
    })
}

/// Get baseline counts (counts before user_metrics_start_date) for cumulative charts
async fn get_baseline_counts(
    pool: &sqlx::PgPool,
    params: &UserCohortAnalysisParams,
) -> Result<BaselineCounts, sqlx::Error> {
    // 1. User registrations before start
    let user_registrations = sqlx::query!(
        "SELECT COUNT(*) as count FROM users WHERE created_at < $1",
        params.user_metrics_start_date
    )
    .fetch_one(pool)
    .await?
    .count
    .unwrap_or(0);
    
    // 2. Paying users before start (billing activations - use trial_ended_at)
    let paying_users = sqlx::query!(
        "SELECT COUNT(*) as count FROM users WHERE trial_ended_at IS NOT NULL AND trial_ended_at < $1 AND subscription_status = 'active'",
        params.user_metrics_start_date
    )
    .fetch_one(pool)
    .await?
    .count
    .unwrap_or(0);
    
    // 3. Total asset creation before start
    let total_asset_creation = sqlx::query!(
        "SELECT COUNT(*) as count FROM assets WHERE created_at < $1",
        params.user_metrics_start_date
    )
    .fetch_one(pool)
    .await?
    .count
    .unwrap_or(0);
    
    // 4. Manual uploads before start (asset_uploaded events)
    let manual_uploads = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM analytics_events
        WHERE event_name = 'asset_uploaded' AND timestamp < $1
        "#,
        params.user_metrics_start_date
    )
    .fetch_one(pool)
    .await?
    .count
    .unwrap_or(0);
    
    // 5. Walkthrough uploads before start - simplified estimate (total assets - manual uploads)
    let walkthrough_uploads = total_asset_creation.saturating_sub(manual_uploads);
    
    // 6. Total uploads  - use total asset creation as proxy
    let total_uploads = total_asset_creation;
    
    // 7-9. Total enhancements before start (simplified - count all asset_enhanced events)
    let total_enhancements = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM analytics_events
        WHERE event_name = 'asset_enhanced' AND timestamp < $1
        "#,
        params.user_metrics_start_date
    )
    .fetch_one(pool)
    .await?
    .count
    .unwrap_or(0);
    
    // For now, we'll use simplified counts for manual vs walkthrough enhancements
    // These can be refined later with proper provenance tracking
    let manual_upload_enhancements = total_enhancements / 2;  // Rough estimate
    let walkthrough_upload_enhancements = total_enhancements / 2;  // Rough estimate
    
    // 10. Walkthrough creation before start
    let walkthrough_creation = sqlx::query!(
        "SELECT COUNT(*) as count FROM vocal_tours WHERE created_at < $1",
        params.user_metrics_start_date
    )
    .fetch_one(pool)
    .await?
    .count
    .unwrap_or(0);
    
    // 11. Document uploads before start
    let document_uploads = sqlx::query!(
        "SELECT COUNT(*) as count FROM documents WHERE created_at < $1",
        params.user_metrics_start_date
    )
    .fetch_one(pool)
    .await?
    .count
    .unwrap_or(0);
    
    // 12. Listing creation before start
    let listing_creation = sqlx::query!(
        "SELECT COUNT(*) as count FROM collections WHERE created_at < $1",
        params.user_metrics_start_date
    )
    .fetch_one(pool)
    .await?
    .count
    .unwrap_or(0);
    
    // 13. Marketing creative usage before start
    let marketing_creative_usage = sqlx::query!(
        "SELECT COUNT(*) as count FROM creatives WHERE created_at < $1",
        params.user_metrics_start_date
    )
    .fetch_one(pool)
    .await?
    .count
    .unwrap_or(0);
    
    Ok(BaselineCounts {
        user_registrations,
        paying_users,
        total_uploads,
        manual_uploads,
        walkthrough_uploads,
        total_enhancements,
        manual_upload_enhancements,
        walkthrough_upload_enhancements,
        total_asset_creation,
        walkthrough_creation,
        document_uploads,
        listing_creation,
        marketing_creative_usage,
    })
}

/// Get comprehensive asset activities with source tracking
async fn get_asset_activities(
    pool: &sqlx::PgPool,
    cohort_user_ids: &[uuid::Uuid],
    params: &UserCohortAnalysisParams,
) -> Result<AssetActivities, sqlx::Error> {
    let total_users = cohort_user_ids.len() as i64;
    
    // A1. Manual Uploads (asset_upload_completed)
    let manual_uploads = get_manual_upload_metrics(pool, cohort_user_ids, params, total_users).await?;
    
    // A2. Walkthrough Uploads (vocal_tour_gennodes_response)
    let walkthrough_uploads = get_walkthrough_upload_metrics(pool, cohort_user_ids, params, total_users).await?;
    
    // A Total. Combined uploads
    let total_uploads = ActivityMetrics {
        users_count: manual_uploads.users_count + walkthrough_uploads.users_count,
        users_percentage: if total_users > 0 {
            ((manual_uploads.users_count + walkthrough_uploads.users_count) as f64 / total_users as f64) * 100.0
        } else { 0.0 },
        total_events: manual_uploads.total_events + walkthrough_uploads.total_events,
        avg_events_per_user: if (manual_uploads.users_count + walkthrough_uploads.users_count) > 0 {
            (manual_uploads.total_events + walkthrough_uploads.total_events) as f64 / (manual_uploads.users_count + walkthrough_uploads.users_count) as f64
        } else { 0.0 },
    };
    
    // B1, B2, B Total. Enhancement metrics with source tracking
    let (manual_upload_enhancements, walkthrough_upload_enhancements, total_enhancements) = 
        get_enhancement_metrics_by_source(pool, cohort_user_ids, params, total_users).await?;
    
    // C. Total Asset Creation (from assets table)
    let total_asset_creation = get_asset_creation_metrics(pool, cohort_user_ids, params, total_users).await?;
    
    Ok(AssetActivities {
        manual_uploads,
        walkthrough_uploads,
        total_uploads,
        manual_upload_enhancements,
        walkthrough_upload_enhancements,
        total_enhancements,
        total_asset_creation,
    })
}

/// Get manual upload metrics (A1)
async fn get_manual_upload_metrics(
    pool: &sqlx::PgPool,
    cohort_user_ids: &[uuid::Uuid],
    params: &UserCohortAnalysisParams,
    total_users: i64,
) -> Result<ActivityMetrics, sqlx::Error> {
    if cohort_user_ids.is_empty() {
        return Ok(ActivityMetrics {
            users_count: 0,
            users_percentage: 0.0,
            total_events: 0,
            avg_events_per_user: 0.0,
        });
    }
    
    let row = sqlx::query!(
        r#"
        SELECT 
            COUNT(DISTINCT user_id) as unique_users,
            COUNT(*) as total_events
        FROM analytics_events
        WHERE event_name = 'asset_upload_completed'
            AND user_id = ANY($1)
            AND timestamp >= $2 AND timestamp <= $3
            AND user_id IS NOT NULL
        "#,
        cohort_user_ids,
        params.analysis_start_date,
        params.analysis_end_date
    )
    .fetch_one(pool)
    .await?;
    
    let users_count = row.unique_users.unwrap_or(0);
    let total_events = row.total_events.unwrap_or(0);
    
    Ok(ActivityMetrics {
        users_count,
        users_percentage: if total_users > 0 { (users_count as f64 / total_users as f64) * 100.0 } else { 0.0 },
        total_events,
        avg_events_per_user: if users_count > 0 { total_events as f64 / users_count as f64 } else { 0.0 },
    })
}

/// Get walkthrough upload metrics (A2) - counts actual assets created, not just events
async fn get_walkthrough_upload_metrics(
    pool: &sqlx::PgPool,
    cohort_user_ids: &[uuid::Uuid],
    params: &UserCohortAnalysisParams,
    total_users: i64,
) -> Result<ActivityMetrics, sqlx::Error> {
    if cohort_user_ids.is_empty() {
        return Ok(ActivityMetrics {
            users_count: 0,
            users_percentage: 0.0,
            total_events: 0,
            avg_events_per_user: 0.0,
        });
    }
    
    // Count actual assets created by walkthrough, not just the number of events
    // Assets can come from two sources:
    // 1. vocal_tour_gennodes_response event's created_asset_ids array
    // 2. vocal_tours table's asset_ids array
    let row = sqlx::query!(
        r#"
        WITH walkthrough_events AS (
            -- Assets from analytics events
            SELECT 
                user_id,
                jsonb_array_elements_text(custom_details->'outcome_metrics'->'created_asset_ids') as asset_id
            FROM analytics_events
            WHERE event_name = 'vocal_tour_gennodes_response'
                AND user_id = ANY($1)
                AND timestamp >= $2 AND timestamp <= $3
                AND user_id IS NOT NULL
        ),
        walkthrough_table AS (
            -- Assets from vocal_tours table (created within analysis period)
            SELECT 
                vt.user_id,
                unnest(vt.asset_ids)::text as asset_id
            FROM vocal_tours vt
            WHERE vt.user_id = ANY($1)
                AND vt.created_at >= $2 AND vt.created_at <= $3
        ),
        all_walkthrough_assets AS (
            SELECT user_id, asset_id FROM walkthrough_events
            UNION
            SELECT user_id, asset_id FROM walkthrough_table
        )
        SELECT 
            COUNT(DISTINCT user_id) as unique_users,
            COUNT(DISTINCT asset_id) as total_assets
        FROM all_walkthrough_assets
        "#,
        cohort_user_ids,
        params.analysis_start_date,
        params.analysis_end_date
    )
    .fetch_one(pool)
    .await?;
    
    let users_count = row.unique_users.unwrap_or(0);
    let total_events = row.total_assets.unwrap_or(0);
    
    Ok(ActivityMetrics {
        users_count,
        users_percentage: if total_users > 0 { (users_count as f64 / total_users as f64) * 100.0 } else { 0.0 },
        total_events,
        avg_events_per_user: if users_count > 0 { total_events as f64 / users_count as f64 } else { 0.0 },
    })
}

/// Get enhancement metrics with source tracking (B1, B2, B Total)
async fn get_enhancement_metrics_by_source(
    pool: &sqlx::PgPool,
    cohort_user_ids: &[uuid::Uuid],
    params: &UserCohortAnalysisParams,
    total_users: i64,
) -> Result<(ActivityMetrics, ActivityMetrics, ActivityMetrics), sqlx::Error> {
    if cohort_user_ids.is_empty() {
        let empty_metrics = ActivityMetrics {
            users_count: 0,
            users_percentage: 0.0,
            total_events: 0,
            avg_events_per_user: 0.0,
        };
        return Ok((empty_metrics.clone(), empty_metrics.clone(), empty_metrics));
    }
    
    // Get all enhancement events for the cohort
    let enhancement_events = sqlx::query!(
        r#"
        SELECT 
            user_id,
            custom_details
        FROM analytics_events
        WHERE event_name = 'asset_enhancement_completed'
            AND user_id = ANY($1)
            AND timestamp >= $2 AND timestamp <= $3
            AND user_id IS NOT NULL
        "#,
        cohort_user_ids,
        params.analysis_start_date,
        params.analysis_end_date
    )
    .fetch_all(pool)
    .await?;
    
    let mut manual_enhancement_users = std::collections::HashSet::new();
    let mut walkthrough_enhancement_users = std::collections::HashSet::new();
    let mut manual_enhancement_count = 0i64;
    let mut walkthrough_enhancement_count = 0i64;
    
    // Process each enhancement event to determine source
    for event in enhancement_events {
        if let Some(user_id) = event.user_id {
            // Parse custom_details to get original_asset_ids
            if let Ok(custom_details) = serde_json::from_value::<serde_json::Value>(event.custom_details) {
                if let Some(original_asset_ids) = custom_details.get("original_asset_ids").and_then(|v| v.as_array()) {
                    for asset_id_val in original_asset_ids {
                        if let Some(asset_id_str) = asset_id_val.as_str() {
                            if let Ok(asset_id) = uuid::Uuid::parse_str(asset_id_str) {
                                // Check if this asset came from manual upload or walkthrough
                                let source = determine_asset_source(pool, asset_id).await?;
                                match source {
                                    AssetSource::Manual => {
                                        manual_enhancement_users.insert(user_id);
                                        manual_enhancement_count += 1;
                                    }
                                    AssetSource::Walkthrough => {
                                        walkthrough_enhancement_users.insert(user_id);
                                        walkthrough_enhancement_count += 1;
                                    }
                                    AssetSource::Unknown => {
                                        // Count as manual by default
                                        manual_enhancement_users.insert(user_id);
                                        manual_enhancement_count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    let manual_users_count = manual_enhancement_users.len() as i64;
    let walkthrough_users_count = walkthrough_enhancement_users.len() as i64;
    let total_enhancement_users = manual_enhancement_users.union(&walkthrough_enhancement_users).count() as i64;
    let total_enhancement_events = manual_enhancement_count + walkthrough_enhancement_count;
    
    let manual_metrics = ActivityMetrics {
        users_count: manual_users_count,
        users_percentage: if total_users > 0 { (manual_users_count as f64 / total_users as f64) * 100.0 } else { 0.0 },
        total_events: manual_enhancement_count,
        avg_events_per_user: if manual_users_count > 0 { manual_enhancement_count as f64 / manual_users_count as f64 } else { 0.0 },
    };
    
    let walkthrough_metrics = ActivityMetrics {
        users_count: walkthrough_users_count,
        users_percentage: if total_users > 0 { (walkthrough_users_count as f64 / total_users as f64) * 100.0 } else { 0.0 },
        total_events: walkthrough_enhancement_count,
        avg_events_per_user: if walkthrough_users_count > 0 { walkthrough_enhancement_count as f64 / walkthrough_users_count as f64 } else { 0.0 },
    };
    
    let total_metrics = ActivityMetrics {
        users_count: total_enhancement_users,
        users_percentage: if total_users > 0 { (total_enhancement_users as f64 / total_users as f64) * 100.0 } else { 0.0 },
        total_events: total_enhancement_events,
        avg_events_per_user: if total_enhancement_users > 0 { total_enhancement_events as f64 / total_enhancement_users as f64 } else { 0.0 },
    };
    
    Ok((manual_metrics, walkthrough_metrics, total_metrics))
}

#[derive(Debug, Clone)]
enum AssetSource {
    Manual,
    Walkthrough,
    Unknown,
}

/// Determine the source of an asset by traversing the entire lineage tree back to the root
async fn determine_asset_source(
    pool: &sqlx::PgPool,
    asset_id: uuid::Uuid,
) -> Result<AssetSource, sqlx::Error> {
    // Use a recursive CTE to traverse the provenance_edges table back to the root asset
    let result = sqlx::query!(
        r#"
        WITH RECURSIVE asset_ancestors AS (
            -- Start with the given asset
            SELECT 
                $1::uuid as asset_id,
                0 as depth
            
            UNION ALL
            
            -- Recursively find ancestor assets through provenance_edges
            SELECT 
                pe.source_id as asset_id,
                aa.depth + 1
            FROM provenance_edges pe
            INNER JOIN asset_ancestors aa ON pe.target_id = aa.asset_id
            WHERE pe.source_type = 'asset' 
                AND pe.target_type = 'asset'
                AND aa.depth < 100  -- Safety limit to prevent infinite loops
        ),
        root_asset AS (
            -- Find the root (the asset with no parent or the deepest ancestor)
            SELECT asset_id
            FROM asset_ancestors
            ORDER BY depth DESC
            LIMIT 1
        )
        SELECT 
            ra.asset_id,
            -- Check if the root asset was from manual upload
            EXISTS (
                SELECT 1
                FROM analytics_events ae
                WHERE ae.event_name = 'asset_upload_completed'
                    AND ae.custom_details->>'asset_id' = ra.asset_id::text
            ) as is_manual_upload,
            -- Check if the root asset was from walkthrough (two sources)
            (
                -- Source 1: From vocal_tour_gennodes_response event
                EXISTS (
                    SELECT 1
                    FROM analytics_events ae
                    WHERE ae.event_name = 'vocal_tour_gennodes_response'
                        AND ae.custom_details->'outcome_metrics'->'created_asset_ids' @> jsonb_build_array(ra.asset_id::text)
                )
                OR
                -- Source 2: From vocal_tours.asset_ids array
                EXISTS (
                    SELECT 1
                    FROM vocal_tours vt
                    WHERE ra.asset_id = ANY(vt.asset_ids)
                )
            ) as is_walkthrough_upload
        FROM root_asset ra
        "#,
        asset_id
    )
    .fetch_optional(pool)
    .await?;
    
    if let Some(row) = result {
        // Check manual upload first (takes precedence)
        if row.is_manual_upload.unwrap_or(false) {
            return Ok(AssetSource::Manual);
        }
        
        // Then check walkthrough
        if row.is_walkthrough_upload.unwrap_or(false) {
            return Ok(AssetSource::Walkthrough);
        }
    }
    
    Ok(AssetSource::Unknown)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Manual verification test for walkthrough asset counting
    /// 
    /// This test verifies that assets created by vocal tour walkthrough are correctly identified.
    /// User a77499f3-8c26-40b6-a0d9-4b3932554154 created a walkthrough on 2025-10-15 that produced 25 assets.
    /// 
    /// The vocal_tour_gennodes_response event shows:
    /// - vocal_tour_id: 2f6b6ae5-8fa0-4571-8843-09b3cb3359a4
    /// - created_asset_ids: 25 assets (f3fc911a-91b8-40b4-8f66-5b1b37441c92, 799123b8-f185-4c9a-a409-d47c283da7f6, etc.)
    /// 
    /// This test should be run manually against production/staging database to verify the fix works.
    /// It's ignored in CI because test databases don't have this production data.
    #[ignore = "Requires production data - run manually against production/staging DB"]
    #[sqlx::test]
    async fn test_walkthrough_assets_counting(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let user_id = uuid::Uuid::parse_str("a77499f3-8c26-40b6-a0d9-4b3932554154").unwrap();
        
        // Get walkthrough upload metrics for this user
        let params = UserCohortAnalysisParams {
            cohort_selection_method: CohortSelectionMethod::ByRegistration,
            cohort_start_date: chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            cohort_end_date: chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            analysis_start_date: chrono::DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z").unwrap().with_timezone(&chrono::Utc),
            analysis_end_date: chrono::DateTime::parse_from_rfc3339("2025-12-31T23:59:59Z").unwrap().with_timezone(&chrono::Utc),
            user_metrics_start_date: chrono::DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z").unwrap().with_timezone(&chrono::Utc),
            user_metrics_end_date: chrono::DateTime::parse_from_rfc3339("2025-12-31T23:59:59Z").unwrap().with_timezone(&chrono::Utc),
        };
        
        let metrics = get_walkthrough_upload_metrics(&pool, &[user_id], &params, 1).await?;
        
        // Should find at least 20 assets from the walkthrough (we know there are 25)
        assert!(
            metrics.total_events >= 20,
            "Expected at least 20 walkthrough assets, but found {}. \
             This user created a vocal tour that produced 25 assets. \
             Check that both vocal_tour_gennodes_response events AND vocal_tours.asset_ids are being counted.",
            metrics.total_events
        );
        
        Ok(())
    }
}

/// Get asset creation metrics from assets table (C)
async fn get_asset_creation_metrics(
    pool: &sqlx::PgPool,
    cohort_user_ids: &[uuid::Uuid],
    params: &UserCohortAnalysisParams,
    total_users: i64,
) -> Result<ActivityMetrics, sqlx::Error> {
    if cohort_user_ids.is_empty() {
        return Ok(ActivityMetrics {
            users_count: 0,
            users_percentage: 0.0,
            total_events: 0,
            avg_events_per_user: 0.0,
        });
    }
    
    let row = sqlx::query!(
        r#"
        SELECT 
            COUNT(DISTINCT user_id) as unique_users,
            COUNT(*) as total_assets
        FROM assets
        WHERE user_id = ANY($1)
            AND created_at >= $2 AND created_at <= $3
            AND user_id IS NOT NULL
        "#,
        cohort_user_ids,
        params.analysis_start_date,
        params.analysis_end_date
    )
    .fetch_one(pool)
    .await?;
    
    let users_count = row.unique_users.unwrap_or(0);
    let total_events = row.total_assets.unwrap_or(0);
    
    Ok(ActivityMetrics {
        users_count,
        users_percentage: if total_users > 0 { (users_count as f64 / total_users as f64) * 100.0 } else { 0.0 },
        total_events,
        avg_events_per_user: if users_count > 0 { total_events as f64 / users_count as f64 } else { 0.0 },
    })
}

/// Get other activities (D, E, F, G)
async fn get_other_activities(
    pool: &sqlx::PgPool,
    cohort_user_ids: &[uuid::Uuid],
    params: &UserCohortAnalysisParams,
) -> Result<OtherActivities, sqlx::Error> {
    let total_users = cohort_user_ids.len() as i64;
    
    // D. Walkthrough Creation (vocal_tours table)
    let walkthrough_creation = get_table_activity_metrics(
        pool, cohort_user_ids, params, total_users, "vocal_tours"
    ).await?;
    
    // E. Document Uploads (documents table)
    let document_uploads = get_table_activity_metrics(
        pool, cohort_user_ids, params, total_users, "documents"
    ).await?;
    
    // F. Listing Creation (collections table)
    let listing_creation = get_table_activity_metrics(
        pool, cohort_user_ids, params, total_users, "collections"
    ).await?;
    
    // G. Marketing Asset Creation (creatives table)
    let marketing_asset_creation = get_creatives_activity_metrics(
        pool, cohort_user_ids, params, total_users
    ).await?;
    
    Ok(OtherActivities {
        walkthrough_creation,
        document_uploads,
        listing_creation,
        marketing_asset_creation,
    })
}

/// Generic function to get activity metrics from a table
async fn get_table_activity_metrics(
    pool: &sqlx::PgPool,
    cohort_user_ids: &[uuid::Uuid],
    params: &UserCohortAnalysisParams,
    total_users: i64,
    table_name: &str,
) -> Result<ActivityMetrics, sqlx::Error> {
    if cohort_user_ids.is_empty() {
        return Ok(ActivityMetrics {
            users_count: 0,
            users_percentage: 0.0,
            total_events: 0,
            avg_events_per_user: 0.0,
        });
    }
    
    let query = format!(
        r#"
        SELECT 
            COUNT(DISTINCT user_id) as unique_users,
            COUNT(*) as total_records
        FROM {}
        WHERE user_id = ANY($1)
            AND created_at >= $2 AND created_at <= $3
            AND user_id IS NOT NULL
        "#,
        table_name
    );
    
    let row = sqlx::query(&query)
        .bind(cohort_user_ids)
        .bind(params.analysis_start_date)
        .bind(params.analysis_end_date)
        .fetch_one(pool)
        .await?;
    
    let users_count: i64 = row.get("unique_users");
    let total_events: i64 = row.get("total_records");
    
    Ok(ActivityMetrics {
        users_count,
        users_percentage: if total_users > 0 { (users_count as f64 / total_users as f64) * 100.0 } else { 0.0 },
        total_events,
        avg_events_per_user: if users_count > 0 { total_events as f64 / users_count as f64 } else { 0.0 },
    })
}

/// Special function for creatives table which doesn't have user_id directly
async fn get_creatives_activity_metrics(
    pool: &sqlx::PgPool,
    cohort_user_ids: &[uuid::Uuid],
    params: &UserCohortAnalysisParams,
    total_users: i64,
) -> Result<ActivityMetrics, sqlx::Error> {
    if cohort_user_ids.is_empty() {
        return Ok(ActivityMetrics {
            users_count: 0,
            users_percentage: 0.0,
            total_events: 0,
            avg_events_per_user: 0.0,
        });
    }
    
    // Creatives don't have user_id directly, need to join with collections
    let row = sqlx::query!(
        r#"
        SELECT 
            COUNT(DISTINCT c.user_id) as unique_users,
            COUNT(cr.*) as total_records
        FROM creatives cr
        LEFT JOIN collections c ON cr.collection_id = c.id
        WHERE c.user_id = ANY($1)
            AND cr.created_at >= $2 AND cr.created_at <= $3
            AND c.user_id IS NOT NULL
        "#,
        cohort_user_ids,
        params.analysis_start_date,
        params.analysis_end_date
    )
    .fetch_one(pool)
    .await?;
    
    let users_count = row.unique_users.unwrap_or(0);
    let total_events = row.total_records.unwrap_or(0);
    
    Ok(ActivityMetrics {
        users_count,
        users_percentage: if total_users > 0 { (users_count as f64 / total_users as f64) * 100.0 } else { 0.0 },
        total_events,
        avg_events_per_user: if users_count > 0 { total_events as f64 / users_count as f64 } else { 0.0 },
    })
}

// ===== TIME SERIES QUERY FUNCTIONS =====

/// Get daily time series for user registrations
async fn get_user_registration_series(
    pool: &sqlx::PgPool,
    cohort_user_ids: &[uuid::Uuid],
    params: &UserCohortAnalysisParams,
) -> Result<Vec<DailyActivityPoint>, sqlx::Error> {
    // Use unified query with empty array for all users case to avoid type mismatch
    let user_ids_param: &[uuid::Uuid] = if cohort_user_ids.is_empty() {
        &[] // Empty array means no user filter in the query logic
    } else {
        cohort_user_ids
    };
    
    let rows = sqlx::query!(
        r#"
        SELECT 
            DATE(created_at) as activity_date,
            COUNT(*) as count
        FROM users
        WHERE (cardinality($1::uuid[]) = 0 OR id = ANY($1))
            AND created_at >= $2 
            AND created_at <= $3
        GROUP BY DATE(created_at)
        ORDER BY activity_date ASC
        "#,
        user_ids_param,
        params.analysis_start_date,
        params.analysis_end_date
    )
    .fetch_all(pool)
    .await?;
    
    Ok(rows.into_iter().map(|row| DailyActivityPoint {
        date: row.activity_date.unwrap(),
        count: row.count.unwrap_or(0),
    }).collect())
}

/// Get daily time series for user logins
async fn get_user_login_series(
    pool: &sqlx::PgPool,
    cohort_user_ids: &[uuid::Uuid],
    params: &UserCohortAnalysisParams,
) -> Result<Vec<DailyActivityPoint>, sqlx::Error> {
    // Use unified query with empty array for all users case to avoid type mismatch
    let user_ids_param: &[uuid::Uuid] = if cohort_user_ids.is_empty() {
        &[] // Empty array means no user filter in the query logic
    } else {
        cohort_user_ids
    };
    
    let rows = sqlx::query!(
        r#"
        SELECT 
            DATE(timestamp) as activity_date,
            COUNT(DISTINCT user_id) as count
        FROM analytics_events
        WHERE event_name = 'user_login_successful'
            AND (cardinality($1::uuid[]) = 0 OR user_id = ANY($1))
            AND timestamp >= $2 
            AND timestamp <= $3
            AND user_id IS NOT NULL
        GROUP BY DATE(timestamp)
        ORDER BY activity_date ASC
        "#,
        user_ids_param,
        params.analysis_start_date,
        params.analysis_end_date
    )
    .fetch_all(pool)
    .await?;
    
    Ok(rows.into_iter().map(|row| DailyActivityPoint {
        date: row.activity_date.unwrap(),
        count: row.count.unwrap_or(0),
    }).collect())
}

/// Get daily time series for paying users (billing activations)
/// Uses trial_ended_at to track when users converted from trial to paid subscription
async fn get_paying_users_series(
    pool: &sqlx::PgPool,
    cohort_user_ids: &[uuid::Uuid],
    params: &UserCohortAnalysisParams,
) -> Result<Vec<DailyActivityPoint>, sqlx::Error> {
    // Use unified query with empty array for all users case to avoid type mismatch
    let user_ids_param: &[uuid::Uuid] = if cohort_user_ids.is_empty() {
        &[] // Empty array means no user filter in the query logic
    } else {
        cohort_user_ids
    };
    
    let rows = sqlx::query!(
        r#"
        SELECT 
            DATE(trial_ended_at) as activity_date,
            COUNT(*) as count
        FROM users
        WHERE (cardinality($1::uuid[]) = 0 OR id = ANY($1))
            AND subscription_status = 'active'
            AND trial_ended_at IS NOT NULL
            AND trial_ended_at >= $2 
            AND trial_ended_at <= $3
        GROUP BY DATE(trial_ended_at)
        ORDER BY activity_date ASC
        "#,
        user_ids_param,
        params.analysis_start_date,
        params.analysis_end_date
    )
    .fetch_all(pool)
    .await?;
    
    Ok(rows.into_iter().map(|row| DailyActivityPoint {
        date: row.activity_date.unwrap(),
        count: row.count.unwrap_or(0),
    }).collect())
}

/// Generic function to get daily time series from analytics events
async fn get_event_daily_series(
    pool: &sqlx::PgPool,
    cohort_user_ids: &[uuid::Uuid],
    params: &UserCohortAnalysisParams,
    event_name: &str,
) -> Result<Vec<DailyActivityPoint>, sqlx::Error> {
    // Use unified query with empty array for all users case to avoid type mismatch
    let user_ids_param: &[uuid::Uuid] = if cohort_user_ids.is_empty() {
        &[] // Empty array means no user filter in the query logic
    } else {
        cohort_user_ids
    };
    
    let rows = sqlx::query!(
        r#"
        SELECT 
            DATE(timestamp) as activity_date,
            COUNT(*) as count
        FROM analytics_events
        WHERE event_name = $1
            AND (cardinality($2::uuid[]) = 0 OR user_id = ANY($2))
            AND timestamp >= $3 
            AND timestamp <= $4
            AND user_id IS NOT NULL
        GROUP BY DATE(timestamp)
        ORDER BY activity_date ASC
        "#,
        event_name,
        user_ids_param,
        params.analysis_start_date,
        params.analysis_end_date
    )
    .fetch_all(pool)
    .await?;
    
    Ok(rows.into_iter().map(|row| DailyActivityPoint {
        date: row.activity_date.unwrap(),
        count: row.count.unwrap_or(0),
    }).collect())
}

/// Generic function to get daily time series from database tables
async fn get_table_daily_series(
    pool: &sqlx::PgPool,
    cohort_user_ids: &[uuid::Uuid],
    params: &UserCohortAnalysisParams,
    table_name: &str,
) -> Result<Vec<DailyActivityPoint>, sqlx::Error> {
    // If cohort_user_ids is empty, query all users (no filter)
    let rows = if cohort_user_ids.is_empty() {
        let query = format!(
            r#"
            SELECT 
                DATE(created_at) as activity_date,
                COUNT(*) as count
            FROM {}
            WHERE created_at >= $1 
                AND created_at <= $2
            GROUP BY DATE(created_at)
            ORDER BY activity_date ASC
            "#,
            table_name
        );
        
        sqlx::query(&query)
            .bind(params.analysis_start_date)
            .bind(params.analysis_end_date)
            .fetch_all(pool)
            .await?
    } else {
        let query = format!(
            r#"
            SELECT 
                DATE(created_at) as activity_date,
                COUNT(*) as count
            FROM {}
            WHERE user_id = ANY($1)
                AND created_at >= $2 
                AND created_at <= $3
                AND user_id IS NOT NULL
            GROUP BY DATE(created_at)
            ORDER BY activity_date ASC
            "#,
            table_name
        );
        
        sqlx::query(&query)
            .bind(cohort_user_ids)
            .bind(params.analysis_start_date)
            .bind(params.analysis_end_date)
            .fetch_all(pool)
            .await?
    };
    
    Ok(rows.into_iter().map(|row| DailyActivityPoint {
        date: row.get("activity_date"),
        count: row.get("count"),
    }).collect())
}

/// Get daily series for creatives (special handling with JOIN)
async fn get_creatives_daily_series(
    pool: &sqlx::PgPool,
    cohort_user_ids: &[uuid::Uuid],
    params: &UserCohortAnalysisParams,
) -> Result<Vec<DailyActivityPoint>, sqlx::Error> {
    // Use unified query with empty array for all users case to avoid type mismatch
    let user_ids_param: &[uuid::Uuid] = if cohort_user_ids.is_empty() {
        &[] // Empty array means no user filter in the query logic
    } else {
        cohort_user_ids
    };
    
    let rows = sqlx::query!(
        r#"
        SELECT 
            DATE(cr.created_at) as activity_date,
            COUNT(*) as count
        FROM creatives cr
        LEFT JOIN collections c ON cr.collection_id = c.id
        WHERE (cardinality($1::uuid[]) = 0 OR c.user_id = ANY($1))
            AND cr.created_at >= $2 
            AND cr.created_at <= $3
        GROUP BY DATE(cr.created_at)
        ORDER BY activity_date ASC
        "#,
        user_ids_param,
        params.analysis_start_date,
        params.analysis_end_date
    )
    .fetch_all(pool)
    .await?;
    
    Ok(rows.into_iter().map(|row| DailyActivityPoint {
        date: row.activity_date.unwrap(),
        count: row.count.unwrap_or(0),
    }).collect())
}

/// Get daily series for walkthrough uploads (counting actual assets)
async fn get_walkthrough_upload_daily_series(
    pool: &sqlx::PgPool,
    cohort_user_ids: &[uuid::Uuid],
    params: &UserCohortAnalysisParams,
) -> Result<Vec<DailyActivityPoint>, sqlx::Error> {
    // Use unified query with empty array for all users case to avoid type mismatch
    let user_ids_param: &[uuid::Uuid] = if cohort_user_ids.is_empty() {
        &[] // Empty array means no user filter in the query logic
    } else {
        cohort_user_ids
    };
    
    let rows = sqlx::query!(
        r#"
        WITH walkthrough_events AS (
            SELECT 
                DATE(timestamp) as activity_date,
                jsonb_array_elements_text(custom_details->'outcome_metrics'->'created_asset_ids') as asset_id
            FROM analytics_events
            WHERE event_name = 'vocal_tour_gennodes_response'
                AND (cardinality($1::uuid[]) = 0 OR user_id = ANY($1))
                AND timestamp >= $2 
                AND timestamp <= $3
                AND user_id IS NOT NULL
        ),
        walkthrough_table AS (
            SELECT 
                DATE(vt.created_at) as activity_date,
                unnest(vt.asset_ids)::text as asset_id
            FROM vocal_tours vt
            WHERE (cardinality($1::uuid[]) = 0 OR vt.user_id = ANY($1))
                AND vt.created_at >= $2 
                AND vt.created_at <= $3
        ),
        all_walkthrough_assets AS (
            SELECT activity_date, asset_id FROM walkthrough_events
            UNION
            SELECT activity_date, asset_id FROM walkthrough_table
        )
        SELECT 
            activity_date,
            COUNT(DISTINCT asset_id) as count
        FROM all_walkthrough_assets
        GROUP BY activity_date
        ORDER BY activity_date ASC
        "#,
        user_ids_param,
        params.analysis_start_date,
        params.analysis_end_date
    )
    .fetch_all(pool)
    .await?;
    
    Ok(rows.into_iter().map(|row| DailyActivityPoint {
        date: row.activity_date.unwrap(),
        count: row.count.unwrap_or(0),
    }).collect())
}

/// Helper function to merge multiple daily series by date
fn merge_daily_series(series_list: Vec<&Vec<DailyActivityPoint>>) -> Vec<DailyActivityPoint> {
    use std::collections::HashMap;
    
    let mut date_map: HashMap<chrono::NaiveDate, i64> = HashMap::new();
    
    for series in series_list {
        for point in series {
            *date_map.entry(point.date).or_insert(0) += point.count;
        }
    }
    
    let mut merged: Vec<DailyActivityPoint> = date_map
        .into_iter()
        .map(|(date, count)| DailyActivityPoint { date, count })
        .collect();
    
    merged.sort_by_key(|p| p.date);
    merged
}

/// Convert ActivityMetrics to ActivityMetricsWithTimeSeries
fn add_time_series_to_metrics(
    metrics: ActivityMetrics,
    daily_series: Vec<DailyActivityPoint>,
) -> ActivityMetricsWithTimeSeries {
    ActivityMetricsWithTimeSeries {
        users_count: metrics.users_count,
        users_percentage: metrics.users_percentage,
        total_events: metrics.total_events,
        avg_events_per_user: metrics.avg_events_per_user,
        daily_series,
    }
}

/// Get comprehensive asset activities with time series
async fn get_asset_activities_with_time_series(
    pool: &sqlx::PgPool,
    cohort_user_ids: &[uuid::Uuid],
    params: &UserCohortAnalysisParams,
) -> Result<AssetActivitiesWithTimeSeries, sqlx::Error> {
    let total_users = cohort_user_ids.len() as i64;
    
    // A1. Manual Uploads with time series
    let manual_uploads_metrics = get_manual_upload_metrics(pool, cohort_user_ids, params, total_users).await?;
    let manual_uploads_series = get_event_daily_series(pool, cohort_user_ids, params, "asset_upload_completed").await?;
    let manual_uploads = add_time_series_to_metrics(manual_uploads_metrics, manual_uploads_series);
    
    // A2. Walkthrough Uploads with time series
    let walkthrough_uploads_metrics = get_walkthrough_upload_metrics(pool, cohort_user_ids, params, total_users).await?;
    let walkthrough_uploads_series = get_walkthrough_upload_daily_series(pool, cohort_user_ids, params).await?;
    let walkthrough_uploads = add_time_series_to_metrics(walkthrough_uploads_metrics, walkthrough_uploads_series);
    
    // A Total. Combined uploads with combined time series
    let total_uploads_series = merge_daily_series(vec![&manual_uploads.daily_series, &walkthrough_uploads.daily_series]);
    let total_uploads = ActivityMetricsWithTimeSeries {
        users_count: manual_uploads.users_count + walkthrough_uploads.users_count,
        users_percentage: if total_users > 0 {
            ((manual_uploads.users_count + walkthrough_uploads.users_count) as f64 / total_users as f64) * 100.0
        } else { 0.0 },
        total_events: manual_uploads.total_events + walkthrough_uploads.total_events,
        avg_events_per_user: if (manual_uploads.users_count + walkthrough_uploads.users_count) > 0 {
            (manual_uploads.total_events + walkthrough_uploads.total_events) as f64 / (manual_uploads.users_count + walkthrough_uploads.users_count) as f64
        } else { 0.0 },
        daily_series: total_uploads_series,
    };
    
    // B1, B2, B Total. Enhancement metrics with time series
    let (manual_upload_enhancements, walkthrough_upload_enhancements, total_enhancements) = 
        get_enhancement_metrics_with_time_series(pool, cohort_user_ids, params, total_users).await?;
    
    // C. Total Asset Creation with time series
    let total_asset_creation_metrics = get_asset_creation_metrics(pool, cohort_user_ids, params, total_users).await?;
    let total_asset_creation_series = get_table_daily_series(pool, cohort_user_ids, params, "assets").await?;
    let total_asset_creation = add_time_series_to_metrics(total_asset_creation_metrics, total_asset_creation_series);
    
    Ok(AssetActivitiesWithTimeSeries {
        manual_uploads,
        walkthrough_uploads,
        total_uploads,
        manual_upload_enhancements,
        walkthrough_upload_enhancements,
        total_enhancements,
        total_asset_creation,
    })
}

/// Get enhancement metrics with time series
async fn get_enhancement_metrics_with_time_series(
    pool: &sqlx::PgPool,
    cohort_user_ids: &[uuid::Uuid],
    params: &UserCohortAnalysisParams,
    total_users: i64,
) -> Result<(ActivityMetricsWithTimeSeries, ActivityMetricsWithTimeSeries, ActivityMetricsWithTimeSeries), sqlx::Error> {
    // Use unified query with empty array for all users case to avoid type mismatch
    let user_ids_param: &[uuid::Uuid] = if cohort_user_ids.is_empty() {
        &[] // Empty array means no user filter in the query logic
    } else {
        cohort_user_ids
    };
    
    // Get all enhancement events with dates
    let enhancement_events = sqlx::query!(
        r#"
        SELECT 
            user_id,
            DATE(timestamp) as event_date,
            custom_details
        FROM analytics_events
        WHERE event_name = 'asset_enhancement_completed'
            AND (cardinality($1::uuid[]) = 0 OR user_id = ANY($1))
            AND timestamp >= $2 AND timestamp <= $3
            AND user_id IS NOT NULL
        "#,
        user_ids_param,
        params.analysis_start_date,
        params.analysis_end_date
    )
    .fetch_all(pool)
    .await?;
    
    let mut manual_enhancement_users = std::collections::HashSet::new();
    let mut walkthrough_enhancement_users = std::collections::HashSet::new();
    let mut manual_enhancement_count = 0i64;
    let mut walkthrough_enhancement_count = 0i64;
    
    // Daily counts by source
    let mut manual_daily_counts: std::collections::HashMap<chrono::NaiveDate, i64> = std::collections::HashMap::new();
    let mut walkthrough_daily_counts: std::collections::HashMap<chrono::NaiveDate, i64> = std::collections::HashMap::new();
    
    // Process each enhancement event
    for event in enhancement_events {
        if let Some(user_id) = event.user_id {
            if let Some(event_date) = event.event_date {
                if let Ok(custom_details) = serde_json::from_value::<serde_json::Value>(event.custom_details) {
                    if let Some(original_asset_ids) = custom_details.get("original_asset_ids").and_then(|v| v.as_array()) {
                        for asset_id_val in original_asset_ids {
                            if let Some(asset_id_str) = asset_id_val.as_str() {
                                if let Ok(asset_id) = uuid::Uuid::parse_str(asset_id_str) {
                                    let source = determine_asset_source(pool, asset_id).await?;
                                    match source {
                                        AssetSource::Manual => {
                                            manual_enhancement_users.insert(user_id);
                                            manual_enhancement_count += 1;
                                            *manual_daily_counts.entry(event_date).or_insert(0) += 1;
                                        }
                                        AssetSource::Walkthrough => {
                                            walkthrough_enhancement_users.insert(user_id);
                                            walkthrough_enhancement_count += 1;
                                            *walkthrough_daily_counts.entry(event_date).or_insert(0) += 1;
                                        }
                                        AssetSource::Unknown => {
                                            manual_enhancement_users.insert(user_id);
                                            manual_enhancement_count += 1;
                                            *manual_daily_counts.entry(event_date).or_insert(0) += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Convert daily counts to series
    let mut manual_series: Vec<DailyActivityPoint> = manual_daily_counts
        .into_iter()
        .map(|(date, count)| DailyActivityPoint { date, count })
        .collect();
    manual_series.sort_by_key(|p| p.date);
    
    let mut walkthrough_series: Vec<DailyActivityPoint> = walkthrough_daily_counts
        .into_iter()
        .map(|(date, count)| DailyActivityPoint { date, count })
        .collect();
    walkthrough_series.sort_by_key(|p| p.date);
    
    let total_series = merge_daily_series(vec![&manual_series, &walkthrough_series]);
    
    let manual_users_count = manual_enhancement_users.len() as i64;
    let walkthrough_users_count = walkthrough_enhancement_users.len() as i64;
    let total_enhancement_users = manual_enhancement_users.union(&walkthrough_enhancement_users).count() as i64;
    let total_enhancement_events = manual_enhancement_count + walkthrough_enhancement_count;
    
    let manual_metrics = ActivityMetricsWithTimeSeries {
        users_count: manual_users_count,
        users_percentage: if total_users > 0 { (manual_users_count as f64 / total_users as f64) * 100.0 } else { 0.0 },
        total_events: manual_enhancement_count,
        avg_events_per_user: if manual_users_count > 0 { manual_enhancement_count as f64 / manual_users_count as f64 } else { 0.0 },
        daily_series: manual_series,
    };
    
    let walkthrough_metrics = ActivityMetricsWithTimeSeries {
        users_count: walkthrough_users_count,
        users_percentage: if total_users > 0 { (walkthrough_users_count as f64 / total_users as f64) * 100.0 } else { 0.0 },
        total_events: walkthrough_enhancement_count,
        avg_events_per_user: if walkthrough_users_count > 0 { walkthrough_enhancement_count as f64 / walkthrough_users_count as f64 } else { 0.0 },
        daily_series: walkthrough_series,
    };
    
    let total_metrics = ActivityMetricsWithTimeSeries {
        users_count: total_enhancement_users,
        users_percentage: if total_users > 0 { (total_enhancement_users as f64 / total_users as f64) * 100.0 } else { 0.0 },
        total_events: total_enhancement_events,
        avg_events_per_user: if total_enhancement_users > 0 { total_enhancement_events as f64 / total_enhancement_users as f64 } else { 0.0 },
        daily_series: total_series,
    };
    
    Ok((manual_metrics, walkthrough_metrics, total_metrics))
}

/// Get other activities with time series
async fn get_other_activities_with_time_series(
    pool: &sqlx::PgPool,
    cohort_user_ids: &[uuid::Uuid],
    params: &UserCohortAnalysisParams,
) -> Result<OtherActivitiesWithTimeSeries, sqlx::Error> {
    let total_users = cohort_user_ids.len() as i64;
    
    // D. Walkthrough Creation
    let walkthrough_creation_metrics = get_table_activity_metrics(
        pool, cohort_user_ids, params, total_users, "vocal_tours"
    ).await?;
    let walkthrough_creation_series = get_table_daily_series(
        pool, cohort_user_ids, params, "vocal_tours"
    ).await?;
    let walkthrough_creation = add_time_series_to_metrics(walkthrough_creation_metrics, walkthrough_creation_series);
    
    // E. Document Uploads
    let document_uploads_metrics = get_table_activity_metrics(
        pool, cohort_user_ids, params, total_users, "documents"
    ).await?;
    let document_uploads_series = get_table_daily_series(
        pool, cohort_user_ids, params, "documents"
    ).await?;
    let document_uploads = add_time_series_to_metrics(document_uploads_metrics, document_uploads_series);
    
    // F. Listing Creation
    let listing_creation_metrics = get_table_activity_metrics(
        pool, cohort_user_ids, params, total_users, "collections"
    ).await?;
    let listing_creation_series = get_table_daily_series(
        pool, cohort_user_ids, params, "collections"
    ).await?;
    let listing_creation = add_time_series_to_metrics(listing_creation_metrics, listing_creation_series);
    
    // G. Marketing Creative Usage (renamed from marketing_asset_creation)
    let marketing_creative_usage_metrics = get_creatives_activity_metrics(
        pool, cohort_user_ids, params, total_users
    ).await?;
    let marketing_creative_usage_series = get_creatives_daily_series(
        pool, cohort_user_ids, params
    ).await?;
    let marketing_creative_usage = add_time_series_to_metrics(marketing_creative_usage_metrics, marketing_creative_usage_series);
    
    Ok(OtherActivitiesWithTimeSeries {
        walkthrough_creation,
        document_uploads,
        listing_creation,
        marketing_creative_usage,
    })
}
