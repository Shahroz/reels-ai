use actix_web::{get, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EnvironmentResponse {
  #[serde(flatten)]
  pub variables: HashMap<String, String>,
}

/// Get environment variables endpoint handler.
/// Returns a plain object of allowed environment variables with their values.
/// Only non-sensitive environment variables are returned for security.
#[utoipa::path(
  get,
  path = "/api/environment",
  tag = "Environment",
  responses(
    (status = 200, description = "Environment variables retrieved successfully", body = EnvironmentResponse),
    (status = 500, description = "Internal server error")
  )
)]
#[get("")]
pub async fn get_environment_variables() -> impl Responder {
  let mut variables = HashMap::new();
  
  // Define allowed environment variables that are safe to expose
  let allowed_keys = [
    "GOOGLE_MAP_API_KEY",
    "GOOGLE_MAP_API_KEY_IOS",
    "LAUNCH_DARKLY_ENV_KEY",
    "LAUNCH_DARKLY_CLIENT_ID",
    "LAUNCH_DARKLY_MOBILE_KEY",
    "DUB_WORKSPACE_ID",
  ];

  // Collect only allowed environment variables
  for (key, value) in env::vars() {
    if allowed_keys.contains(&key.as_str()) {
      variables.insert(key, value);
    }
  }

  let response = EnvironmentResponse { variables };

  HttpResponse::Ok().json(response)
} 