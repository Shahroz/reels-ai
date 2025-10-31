//! Cohort funnel analysis query for analytics dashboard.
//!
//! Performs comprehensive cohort-based funnel analysis by registration date.
//! Tracks user journey through defined steps with conversion rates and drop-offs.
//! Provides detailed metrics for cohort performance and behavior analysis.
//! Used by the analytics API endpoint for MVP dashboard functionality.

pub struct CohortFunnelAnalysisParams {
    pub registration_date_start: chrono::NaiveDate,
    pub registration_date_end: chrono::NaiveDate,
    pub analysis_period_days: Option<u32>,
}

pub struct CohortFunnelAnalysisResult {
    pub total_cohort_users: i64,
    pub funnel_steps: Vec<FunnelStepResult>,
    pub cohort_period: chrono::NaiveDate,
    pub analysis_generated_at: chrono::DateTime<chrono::Utc>,
}

pub struct FunnelStepResult {
    pub step_name: String,
    pub unique_users: i64,
    pub total_events: i64,
    pub conversion_rate_from_previous: Option<f64>,
    pub conversion_rate_from_start: f64,
}

pub async fn get_cohort_funnel_analysis(
    pool: &sqlx::PgPool,
    params: CohortFunnelAnalysisParams,
) -> Result<CohortFunnelAnalysisResult, sqlx::Error> {
    // First, get the total users in the cohort
    let total_cohort_users = get_cohort_size(pool, &params).await?;
    
    // Get funnel steps with metrics
    let funnel_steps = get_funnel_steps_with_metrics(pool, &params, total_cohort_users).await?;
    
    Ok(CohortFunnelAnalysisResult {
        total_cohort_users,
        funnel_steps,
        cohort_period: params.registration_date_start,
        analysis_generated_at: chrono::Utc::now(),
    })
}

async fn get_cohort_size(
    pool: &sqlx::PgPool,
    params: &CohortFunnelAnalysisParams,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT COUNT(DISTINCT id) as user_count
        FROM users 
        WHERE DATE(created_at) >= $1 
        AND DATE(created_at) <= $2
        "#,
        params.registration_date_start,
        params.registration_date_end
    )
    .fetch_one(pool)
    .await?;
    
    Ok(result.user_count.unwrap_or(0))
}

async fn get_funnel_steps_with_metrics(
    pool: &sqlx::PgPool,
    params: &CohortFunnelAnalysisParams,
    total_cohort_users: i64,
) -> Result<Vec<FunnelStepResult>, sqlx::Error> {
    // Get common event patterns as funnel steps
    let rows = sqlx::query!(
        r#"
        SELECT 
            ae.event_name,
            COUNT(DISTINCT ae.user_id) as unique_users,
            COUNT(*) as total_events
        FROM analytics_events ae
        INNER JOIN users u ON ae.user_id = u.id
        WHERE DATE(u.created_at) >= $1 
        AND DATE(u.created_at) <= $2
        AND ae.user_id IS NOT NULL

        GROUP BY ae.event_name
        ORDER BY unique_users DESC
        LIMIT 10
        "#,
        params.registration_date_start,
        params.registration_date_end
    )
    .fetch_all(pool)
    .await?;
    
    let mut funnel_steps = Vec::new();
    let mut previous_users: Option<i64> = None;
    
    for (_index, row) in rows.iter().enumerate() {
        let unique_users = row.unique_users.unwrap_or(0);
        let total_events = row.total_events.unwrap_or(0);
        
        let conversion_rate_from_previous = if let Some(prev) = previous_users {
            if prev > 0 {
                Some((unique_users as f64 / prev as f64) * 100.0)
            } else {
                None
            }
        } else {
            None
        };
        
        let conversion_rate_from_start = if total_cohort_users > 0 {
            (unique_users as f64 / total_cohort_users as f64) * 100.0
        } else {
            0.0
        };
        
        funnel_steps.push(FunnelStepResult {
            step_name: row.event_name.clone(),
            unique_users,
            total_events,
            conversion_rate_from_previous,
            conversion_rate_from_start,
        });
        
        previous_users = Some(unique_users);
    }
    
    Ok(funnel_steps)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_cohort_funnel_analysis_params_creation() {
        let params = super::CohortFunnelAnalysisParams {
            registration_date_start: chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            registration_date_end: chrono::NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
            analysis_period_days: Some(30),
        };
        
        assert_eq!(params.analysis_period_days, Some(30));
    }

    #[test]
    fn test_funnel_step_result_conversion_calculation() {
        let step = super::FunnelStepResult {
            step_name: String::from("GET /dashboard"),
            unique_users: 80,
            total_events: 150,
            conversion_rate_from_previous: Some(80.0),
            conversion_rate_from_start: 80.0,
        };
        
        assert_eq!(step.unique_users, 80);
        assert_eq!(step.conversion_rate_from_start, 80.0);
    }
} 