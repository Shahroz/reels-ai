//! Database model for watermarking jobs.
//!
//! This module defines the WatermarkingJob struct representing asynchronous watermarking operations.
//! Jobs track the status of watermark application processes and store configuration data.
//! This enables async processing for larger images in Phase 2 while supporting sync operations in Phase 1.

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct WatermarkingJob {
    #[schema(format = "uuid", value_type = String)]
    pub id: uuid::Uuid,
    #[schema(format = "uuid", value_type = String)]
    pub user_id: uuid::Uuid,
    #[schema(format = "uuid", value_type = String)]
    pub source_asset_id: uuid::Uuid,
    #[schema(format = "uuid", value_type = String)]
    pub logo_asset_id: uuid::Uuid,
    pub watermark_config: serde_json::Value,
    pub status: std::string::String,
    #[schema(format = "uuid", value_type = String, nullable = true)]
    pub result_asset_id: std::option::Option<uuid::Uuid>,
    pub error_message: std::option::Option<std::string::String>,
    pub processing_time_ms: std::option::Option<i32>,
    #[schema(format = "date-time", value_type = String)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[schema(format = "date-time", value_type = String)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Watermarking job status enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub enum JobStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "processing")]
    Processing,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "pending"),
            JobStatus::Processing => write!(f, "processing"),
            JobStatus::Completed => write!(f, "completed"),
            JobStatus::Failed => write!(f, "failed"),
        }
    }
}

impl std::str::FromStr for JobStatus {
    type Err = std::string::String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "pending" => std::result::Result::Ok(JobStatus::Pending),
            "processing" => std::result::Result::Ok(JobStatus::Processing),
            "completed" => std::result::Result::Ok(JobStatus::Completed),
            "failed" => std::result::Result::Ok(JobStatus::Failed),
            _ => std::result::Result::Err(std::format!("Invalid job status: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::JobStatus;

    #[test]
    fn test_job_status_string_conversion() {
        // Test Display trait implementation
        assert_eq!(JobStatus::Pending.to_string(), "pending");
        assert_eq!(JobStatus::Processing.to_string(), "processing");
        assert_eq!(JobStatus::Completed.to_string(), "completed");
        assert_eq!(JobStatus::Failed.to_string(), "failed");
    }

    #[test]
    fn test_job_status_parsing() {
        // Test FromStr trait implementation
        assert!(matches!("pending".parse::<JobStatus>(), std::result::Result::Ok(JobStatus::Pending)));
        assert!(matches!("processing".parse::<JobStatus>(), std::result::Result::Ok(JobStatus::Processing)));
        assert!(matches!("completed".parse::<JobStatus>(), std::result::Result::Ok(JobStatus::Completed)));
        assert!(matches!("failed".parse::<JobStatus>(), std::result::Result::Ok(JobStatus::Failed)));
        
        // Test error case
        let result = "invalid_status".parse::<JobStatus>();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid job status: invalid_status");
    }

    #[test]
    fn test_job_status_roundtrip() {
        // Test that Display and FromStr are consistent
        let statuses = [JobStatus::Pending, JobStatus::Processing, JobStatus::Completed, JobStatus::Failed];
        
        for status in statuses {
            let status_str = status.to_string();
            let parsed_status: JobStatus = status_str.parse().unwrap();
            assert!(matches!((status, parsed_status), 
                (JobStatus::Pending, JobStatus::Pending) |
                (JobStatus::Processing, JobStatus::Processing) |
                (JobStatus::Completed, JobStatus::Completed) |
                (JobStatus::Failed, JobStatus::Failed)
            ));
        }
    }
}
