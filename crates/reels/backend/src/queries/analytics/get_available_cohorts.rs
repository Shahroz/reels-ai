//! Available cohorts query for analytics dashboard.
//!
//! Retrieves user registration cohorts grouped by date for selection.
//! Shows user count per registration date to help with cohort analysis.
//! Provides data for cohort selector interface in the analytics dashboard.
//! Orders by registration date for chronological cohort visualization.

pub struct AvailableCohort {
    pub registration_date: chrono::NaiveDate,
    pub user_count: i64,
    pub first_activity_date: Option<chrono::NaiveDate>,
    pub last_activity_date: Option<chrono::NaiveDate>,
}

pub async fn get_available_cohorts(
    pool: &sqlx::PgPool,
    limit: Option<i32>,
) -> Result<Vec<AvailableCohort>, sqlx::Error> {
    let limit_value = limit.unwrap_or(100) as i64;
    
    let rows = sqlx::query!(
        r#"
        SELECT 
            DATE(u.created_at) as registration_date,
            COUNT(*) as user_count,
            MIN(DATE(ae.timestamp)) as first_activity_date,
            MAX(DATE(ae.timestamp)) as last_activity_date
        FROM users u
        LEFT JOIN analytics_events ae ON u.id = ae.user_id
        GROUP BY DATE(u.created_at)
        HAVING COUNT(*) > 0
        ORDER BY DATE(u.created_at) DESC
        LIMIT $1
        "#,
        limit_value
    )
    .fetch_all(pool)
    .await?;
    
    let cohorts = rows.into_iter().map(|row| {
        AvailableCohort {
            registration_date: row.registration_date.unwrap_or_else(|| chrono::Utc::now().date_naive()),
            user_count: row.user_count.unwrap_or(0),
            first_activity_date: row.first_activity_date,
            last_activity_date: row.last_activity_date,
        }
    }).collect();
    
    Ok(cohorts)
}

pub async fn get_cohort_by_date(
    pool: &sqlx::PgPool,
    registration_date: chrono::NaiveDate,
) -> Result<Option<AvailableCohort>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT 
            DATE(u.created_at) as registration_date,
            COUNT(*) as user_count,
            MIN(DATE(ae.timestamp)) as first_activity_date,
            MAX(DATE(ae.timestamp)) as last_activity_date
        FROM users u
        LEFT JOIN analytics_events ae ON u.id = ae.user_id
        WHERE DATE(u.created_at) = $1
        GROUP BY DATE(u.created_at)
        "#,
        registration_date
    )
    .fetch_optional(pool)
    .await?;
    
    Ok(row.map(|r| AvailableCohort {
        registration_date: r.registration_date.unwrap_or(registration_date),
        user_count: r.user_count.unwrap_or(0),
        first_activity_date: r.first_activity_date,
        last_activity_date: r.last_activity_date,
    }))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_available_cohort_creation() {
        let cohort = super::AvailableCohort {
            registration_date: chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            user_count: 50,
            first_activity_date: Some(chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()),
            last_activity_date: Some(chrono::NaiveDate::from_ymd_opt(2025, 1, 20).unwrap()),
        };
        
        assert_eq!(cohort.user_count, 50);
        assert!(cohort.first_activity_date.is_some());
    }

    #[test]
    fn test_cohort_date_handling() {
        let registration_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        
        // Test date comparison
        let later_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 2).unwrap();
        assert!(later_date > registration_date);
    }
} 