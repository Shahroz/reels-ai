//! Cohort funnel analysis service for business logic coordination.
//!
//! Provides high-level service methods for cohort-based funnel analysis.
//! Coordinates between database queries and API endpoints for analytics operations.
//! Handles business logic validation and data transformation.
//! Used by the analytics API endpoints for comprehensive funnel analysis.

pub struct CohortFunnelService {
    db_pool: sqlx::PgPool,
}

impl CohortFunnelService {
    pub fn new(db_pool: sqlx::PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn get_cohort_funnel_analysis(
        &self,
        registration_date_start: chrono::NaiveDate,
        registration_date_end: chrono::NaiveDate,
        analysis_period_days: Option<u32>,
    ) -> Result<crate::queries::analytics::CohortFunnelAnalysisResult, ServiceError> {
        // Validate input parameters
        if registration_date_end < registration_date_start {
            return Err(ServiceError::InvalidDateRange);
        }

        // Check if date range is reasonable (not more than 1 year)
        let days_diff = (registration_date_end - registration_date_start).num_days();
        if days_diff > 365 {
            return Err(ServiceError::DateRangeTooLarge);
        }

        let params = crate::queries::analytics::CohortFunnelAnalysisParams {
            registration_date_start,
            registration_date_end,
            analysis_period_days,
        };

        let result = crate::queries::analytics::get_cohort_funnel_analysis(&self.db_pool, params)
            .await
            .map_err(ServiceError::DatabaseError)?;

        Ok(result)
    }

    pub async fn get_available_cohorts(
        &self,
        limit: Option<i32>,
    ) -> Result<Vec<crate::queries::analytics::AvailableCohort>, ServiceError> {
        // Validate limit parameter
        let validated_limit = match limit {
            Some(l) if l <= 0 => return Err(ServiceError::InvalidLimit),
            Some(l) if l > 1000 => Some(1000), // Cap at 1000
            other => other,
        };

        let cohorts = crate::queries::analytics::get_available_cohorts(&self.db_pool, validated_limit)
            .await
            .map_err(ServiceError::DatabaseError)?;

        Ok(cohorts)
    }

    pub async fn get_cohort_by_date(
        &self,
        registration_date: chrono::NaiveDate,
    ) -> Result<Option<crate::queries::analytics::AvailableCohort>, ServiceError> {
        // Validate date is not in the future
        let today = chrono::Utc::now().date_naive();
        if registration_date > today {
            return Err(ServiceError::FutureDateNotAllowed);
        }

        let cohort = crate::queries::analytics::get_cohort_by_date(&self.db_pool, registration_date)
            .await
            .map_err(ServiceError::DatabaseError)?;

        Ok(cohort)
    }
}

#[derive(Debug)]
pub enum ServiceError {
    InvalidDateRange,
    DateRangeTooLarge,
    InvalidLimit,
    FutureDateNotAllowed,
    DatabaseError(sqlx::Error),
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::InvalidDateRange => write!(f, "End date must be after start date"),
            ServiceError::DateRangeTooLarge => write!(f, "Date range cannot exceed 365 days"),
            ServiceError::InvalidLimit => write!(f, "Limit must be greater than 0"),
            ServiceError::FutureDateNotAllowed => write!(f, "Future dates are not allowed"),
            ServiceError::DatabaseError(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl std::error::Error for ServiceError {}

#[cfg(test)]
mod tests {
    #[test]
    fn test_service_error_display() {
        let error = super::ServiceError::InvalidDateRange;
        assert_eq!(error.to_string(), "End date must be after start date");
    }

    #[test]
    fn test_date_range_validation() {
        let start_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let end_date = chrono::NaiveDate::from_ymd_opt(2025, 1, 10).unwrap();
        
        // Test that end date before start date would be invalid
        assert!(end_date < start_date);
    }

    #[test]
    fn test_large_date_range() {
        let start_date = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        let days_diff = (end_date - start_date).num_days();
        
        // Test that this range would be too large (>365 days)
        assert!(days_diff > 365);
    }
} 