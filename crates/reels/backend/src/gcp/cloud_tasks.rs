//! Manages interactions with Google Cloud Tasks.
//!
//! This module provides a client for creating tasks, specifically for one-time
//! background jobs. It handles the construction of task requests with OIDC
//! authentication.

use anyhow::{Context, Result};
use bytes::Bytes;
use google_cloud_tasks_v2::{
    client::CloudTasks,
    model::{HttpMethod, HttpRequest, OidcToken, Task},
};
use tracing::instrument;

/// A client for managing Google Cloud Tasks.
#[derive(Clone)]
pub struct CloudTasksClient {
    client: CloudTasks,
    project_id: String,
    location: String,
    queue_id: String,
    backend_url: String,
}

impl CloudTasksClient {
    /// Creates a new `CloudTasksClient`.
    ///
    /// This function initializes the `CloudTasks` client and reads necessary
    /// configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if the `CloudTasks` client cannot be created or if
    /// required environment variables are not set.
    #[instrument(name = "cloud_tasks_client::new", skip_all)]
    pub async fn new() -> Result<Self> {
        let client = CloudTasks::builder()
            .build()
            .await
            .context("Failed to create Google Cloud Tasks client")?;

        let project_id = std::env::var("GCP_PROJECT_ID")
            .context("GCP_PROJECT_ID environment variable not set")?;
        let location =
            std::env::var("GCP_LOCATION").context("GCP_LOCATION environment variable not set")?;
        let queue_id = std::env::var("GCP_TASKS_QUEUE_ID")
            .context("GCP_TASKS_QUEUE_ID environment variable not set")?;
        let backend_url = std::env::var("BACKEND_URL")
            .context("BACKEND_URL environment variable not set. This is required to construct the target URL for tasks.")?;

        Ok(Self {
            client,
            project_id,
            location,
            queue_id,
            backend_url,
        })
    }

    /// Returns the full parent path for tasks in the configured queue.
    fn queue_path(&self) -> String {
        format!(
            "projects/{}/locations/{}/queues/{}",
            self.project_id, self.location, self.queue_id
        )
    }

    /// Returns the full resource name for a specific task.
    fn task_name_path(&self, task_id: &str) -> String {
        format!("{}/tasks/{}", self.queue_path(), task_id)
    }

    /// Creates a new HTTP task in the configured queue.
    #[instrument(skip(self, body, service_account_email), fields(task_id = task_id, relative_uri = relative_uri))]
    pub async fn create_http_task(
        &self,
        task_id: &str,
        relative_uri: &str,
        body: Bytes,
        service_account_email: &str,
    ) -> Result<Task> {
        let target_uri = format!("{}{}", self.backend_url, relative_uri);

        let oidc_token = OidcToken::new()
            .set_service_account_email(service_account_email.to_string())
            .set_audience(target_uri.clone());

        let http_request = HttpRequest::new()
            .set_url(target_uri)
            .set_http_method(HttpMethod::Post)
            .set_body(body)
            .set_oidc_token(oidc_token);

        let task = Task::new()
            .set_name(self.task_name_path(task_id))
            .set_http_request(http_request);

        let created_task = self
            .client
            .create_task()
            .set_parent(self.queue_path())
            .set_task(task)
            .send()
            .await
            .context(format!("Failed to create cloud task '{task_id}'"))?;

        Ok(created_task)
    }

    /// Creates a new HTTP task with a custom JWT Bearer token for auth.
    /// This is used for internal task authentication where OIDC is not suitable.
    #[instrument(skip(self, body, jwt, extra_headers), fields(task_id = task_id, relative_uri = relative_uri))]
    pub async fn create_http_task_with_jwt(
        &self,
        task_id: &str,
        relative_uri: &str,
        body: Bytes,
        jwt: &str,
        extra_headers: Option<&std::collections::HashMap<String, String>>,
    ) -> Result<Task> {
        let target_uri = format!("{}{}", self.backend_url, relative_uri);

        let mut headers = std::collections::HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {jwt}"),
        );
        headers.insert(
            "Content-Type".to_string(),
            "application/json".to_string(),
        );

        // Merge any extra headers provided
        if let Some(extra) = extra_headers {
            for (key, value) in extra.iter() {
                headers.insert(key.clone(), value.clone());
            }
        }

        let mut http_request = HttpRequest::new()
            .set_url(target_uri)
            .set_http_method(HttpMethod::Post)
            .set_body(body);

        http_request.headers = headers;

        let task = Task::new()
            .set_name(self.task_name_path(task_id))
            .set_http_request(http_request);

        let created_task = self
            .client
            .create_task()
            .set_parent(self.queue_path())
            .set_task(task)
            .send()
            .await
            .context(format!("Failed to create cloud task with jwt '{task_id}'"))?;

        Ok(created_task)
    }

    /// Deletes a task from a GCP queue by its full name.
    #[instrument(skip(self), fields(task_name = task_name))]
    pub async fn delete_task(&self, task_name: &str) -> Result<()> {
        self.client
            .delete_task()
            .set_name(task_name.to_string())
            .send()
            .await
            .context(format!("Failed to delete cloud task '{task_name}'"))?;

        Ok(())
    }
}
