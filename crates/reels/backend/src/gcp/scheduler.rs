//! Manages interactions with Google Cloud Scheduler.
//!
//! This module provides a client for creating, updating, and deleting scheduler jobs,
//! specifically for the "Infinite Research" feature. It handles the construction of
//! job requests with custom JWT authentication headers.

use google_cloud_wkt::FieldMask;
use anyhow::{Context, Result};
use google_cloud_scheduler_v1::{
    client::CloudScheduler,
    model::{job::Target, HttpMethod, HttpTarget, Job},
};
use std::collections::HashMap;
use tracing::instrument;

/// A client for managing Google Cloud Scheduler jobs.
#[derive(Clone)]
pub struct SchedulerClient {
    client: CloudScheduler,
    project_id: String,
    location: String,
    backend_url: String,
}

impl SchedulerClient {
    /// Creates a new `SchedulerClient`.
    ///
    /// This function initializes the `CloudScheduler` client and reads necessary
    /// configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if the `CloudScheduler` client cannot be created or if
    /// required environment variables (`GCP_PROJECT_ID`, `GCP_LOCATION`, `BACKEND_URL`) are not set.
    #[instrument(name = "scheduler_client::new", skip_all)]
    pub async fn new() -> Result<Self> {
        let client = CloudScheduler::builder()
            .build()
            .await
            .context("Failed to create Google Cloud Scheduler client")?;

        let project_id = std::env::var("GCP_PROJECT_ID")
            .context("GCP_PROJECT_ID environment variable not set")?;
        let location =
            std::env::var("GCP_LOCATION").context("GCP_LOCATION environment variable not set")?;
        let backend_url = std::env::var("BACKEND_URL")
            .context("BACKEND_URL environment variable not set. This is required to construct the target URL for scheduler jobs.")?;

        Ok(Self {
            client,
            project_id,
            location,
            backend_url,
        })
    }

    /// Returns the full parent path for scheduler jobs.
    fn parent_path(&self) -> String {
        format!(
            "projects/{}/locations/{}",
            self.project_id, self.location
        )
    }

    /// Returns the full resource name for a specific job.
    fn job_name_path(&self, job_id: &str) -> String {
        format!("{}/jobs/{}", self.parent_path(), job_id)
    }

    /// Creates a new scheduler job.
    #[instrument(skip(self, jwt_token), fields(job_id = job_id, cron_schedule = cron_schedule))]
    pub async fn create_scheduler_job(
        &self,
        job_id: &str,
        description: &str,
        cron_schedule: &str,
        time_zone: &str,
        jwt_token: &str,
    ) -> Result<String> {
        let target_uri = format!(
            "{}/api/internal/run-infinite-research/{}",
            self.backend_url, job_id
        );

        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {jwt_token}"),
        );
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let http_target = HttpTarget::new()
            .set_uri(target_uri)
            .set_http_method(HttpMethod::Post)
            .set_headers(headers);

        let job = Job::new()
            .set_name(self.job_name_path(job_id))
            .set_description(description.to_string())
            .set_schedule(cron_schedule.to_string())
            .set_time_zone(time_zone.to_string())
            .set_target(Target::HttpTarget(Box::new(http_target)));

        let created_job = self
            .client
            .create_job()
            .set_parent(self.parent_path())
            .set_job(job)
            .send()
            .await
            .context(format!("Failed to create scheduler job '{job_id}'"))?;

        Ok(created_job.name)
    }

    /// Updates an existing scheduler job.
    #[instrument(skip(self, jwt_token), fields(job_id = job_id, cron_schedule = cron_schedule))]
    pub async fn update_scheduler_job(
        &self,
        job_id: &str,
        description: &str,
        cron_schedule: &str,
        time_zone: &str,
        jwt_token: &str,
    ) -> Result<String> {
        let target_uri = format!(
            "{}/api/internal/run-infinite-research/{}",
            self.backend_url, job_id
        );

        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {jwt_token}"),
        );
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let http_target = HttpTarget::new()
            .set_uri(target_uri)
            .set_http_method(HttpMethod::Post)
            .set_headers(headers);

        let job = Job::new()
            .set_name(self.job_name_path(job_id))
            .set_description(description.to_string())
            .set_schedule(cron_schedule.to_string())
            .set_time_zone(time_zone.to_string())
            .set_target(Target::HttpTarget(Box::new(http_target)));

        let updated_job = self
            .client
            .update_job()
            .set_job(job)
            .set_update_mask(FieldMask::default().set_paths(vec![
                "description".to_string(),
                "schedule".to_string(),
                "time_zone".to_string(),
                "http_target".to_string(),
            ]))
            .send()
            .await
            .context(format!("Failed to update scheduler job '{job_id}'"))?;

       Ok(updated_job.name)
   }

    /// Deletes a scheduler job.
    #[instrument(skip(self), fields(job_id = job_id))]
    pub async fn delete_scheduler_job(&self, job_id: &str) -> Result<()> {
        self.client
            .delete_job()
            .set_name(self.job_name_path(job_id))
            .send()
            .await
            .context(format!("Failed to delete scheduler job '{job_id}'"))?;

        Ok(())
    }

    /// Pauses a scheduler job.
    #[instrument(skip(self), fields(job_id = job_id))]
    pub async fn pause_scheduler_job(&self, job_id: &str) -> Result<()> {
        self.client
            .pause_job()
            .set_name(self.job_name_path(job_id))
            .send()
            .await
            .context(format!("Failed to pause scheduler job '{job_id}'"))?;
        Ok(())
    }

    /// Resumes a scheduler job.
    #[instrument(skip(self), fields(job_id = job_id))]
    pub async fn resume_scheduler_job(&self, job_id: &str) -> Result<()> {
        self.client
            .resume_job()
            .set_name(self.job_name_path(job_id))
            .send()
            .await
            .context(format!("Failed to resume scheduler job '{job_id}'"))?;
        Ok(())
    }

    /// Forces a job to run immediately.
    #[instrument(skip(self), fields(job_id = job_id))]
    pub async fn run_scheduler_job(&self, job_id: &str) -> Result<()> {
        self.client
            .run_job()
            .set_name(self.job_name_path(job_id))
            .send()
            .await
            .context(format!("Failed to force run scheduler job '{job_id}'"))?;

        Ok(())
    }
}
