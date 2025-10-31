use anyhow::anyhow;
use async_trait::async_trait;
use gcp_auth::AuthenticationManager;
use gcp_bigquery_client::auth::Authenticator;
use gcp_bigquery_client::error::BQError;
use google_cloud_token::{TokenSource, TokenSourceProvider};
use std::sync::{Arc};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::instrument;

const GCP_API_AUTH_SCOPE: &str = "https://www.googleapis.com/auth/cloud-platform";

#[derive(Debug, Clone)]
pub struct GCPTokenSource {
    inner: Arc<Mutex<Option<TokenData>>>,
}

impl Default for GCPTokenSource {
    fn default() -> Self {
        Self::new()
    }
}

impl GCPTokenSource {
    pub fn new() -> Self {
        // lazy initialization
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }
}

#[derive(Clone, Debug)]
struct TokenData {
    token: String,
    expires_at: Instant,
}

impl TokenSourceProvider for GCPTokenSource {
    fn token_source(&self) -> Arc<dyn TokenSource> {
        Arc::new(self.clone())
    }
}

#[async_trait]
impl Authenticator for GCPTokenSource {
    #[instrument(skip(self))]
    async fn access_token(&self) -> Result<String, BQError> {
        let mut inner = self.inner.lock().await;

        // create or refresh
        match inner.as_mut() {
            None => {
                *inner = Some(get_gcp_token_data().await.map_err(|_e| BQError::NoToken)?);
            }
            Some(inner_data) => {
                if Instant::now() >= inner_data.expires_at {
                    // Token expired, refresh
                    *inner = Some(get_gcp_token_data().await.map_err(|_e| BQError::NoToken)?);
                }
            }
        }

        Ok(inner.as_ref().ok_or(BQError::NoToken)?.token.clone())
    }
}

#[async_trait]
impl TokenSource for GCPTokenSource {
    #[instrument(skip(self))]
    async fn token(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut inner = self.inner.lock().await;

        match inner.as_mut() {
            None => {
                *inner = Some(get_gcp_token_data().await?);
            }
            Some(inner_data) => {
                if Instant::now() >= inner_data.expires_at {
                    *inner = Some(get_gcp_token_data().await?);
                }
            }
        }

        let token = inner
            .as_ref()
            .ok_or("No token available")?
            .token
            .clone();

        Ok(format!("Bearer {token}"))
    }
}

#[instrument]
pub async fn get_gcp_authn_token() -> anyhow::Result<String> {
    let authentication_manager = AuthenticationManager::new()
        .await
        .map_err(|e| anyhow!(format!("Failed to create AuthenticationManager: {}", e)))?;
    let scopes = &[GCP_API_AUTH_SCOPE];
    let token = authentication_manager
        .get_token(scopes)
        .await
        .map_err(|e| anyhow!(format!("Failed to get token: {}", e)))?;
    Ok(token.as_str().to_string())
}

#[instrument]
pub async fn get_gcp_token_data() -> anyhow::Result<TokenData> {
    let token = get_gcp_authn_token().await?;
    let expires_at = Instant::now() + Duration::from_secs(3600 - 60);
    Ok(TokenData { token, expires_at })
}
