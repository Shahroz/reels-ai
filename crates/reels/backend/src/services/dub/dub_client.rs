//! HTTP client for Dub API integration
//!
//! This module provides the HTTP client for communicating with Dub's REST API
//! for lead and sale event tracking.

use anyhow::{Context, Result};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{instrument, warn};

use super::dub_config::DubConfig;
use super::dub_service_trait::{DubEventResponse, DubLeadEvent, DubSaleEvent};

/// HTTP client for Dub API
#[derive(Debug, Clone)]
pub struct DubClient {
    client: Client,
    config: DubConfig,
}

/// Request payload for Dub lead tracking endpoint
#[derive(Debug, Serialize)]
struct DubLeadRequest {
    #[serde(rename = "customerId")]
    customer_id: String,
    #[serde(rename = "customerEmail")]
    customer_email: String,
    #[serde(rename = "eventName")]
    event_name: String,
    #[serde(rename = "clickId", skip_serializing_if = "Option::is_none")]
    click_id: Option<String>,
    #[serde(flatten)]
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Request payload for Dub sale tracking endpoint
#[derive(Debug, Serialize)]
struct DubSaleRequest {
    #[serde(rename = "customerId")]
    customer_id: String,
    #[serde(rename = "amount")]
    amount: f64,
    currency: String,
    #[serde(rename = "eventName")]
    event_name: String,
    #[serde(rename = "clickId", skip_serializing_if = "Option::is_none")]
    click_id: Option<String>,
    #[serde(flatten)]
    metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Request payload for Dub referral token generation
#[derive(Debug, Serialize)]
struct DubReferralTokenRequest {
    #[serde(rename = "tenantId")]
    tenant_id: String,
    partner: DubPartnerInfo,
}

/// Partner information for Dub referral token
#[derive(Debug, Serialize)]
struct DubPartnerInfo {
    name: String,
    email: String,
    image: Option<String>,
    #[serde(rename = "tenantId")]
    tenant_id: String,
    #[serde(rename = "groupId", skip_serializing_if = "Option::is_none")]
    group_id: Option<String>,
}

/// Response from Dub referral token endpoint
#[derive(Debug, Deserialize)]
struct DubReferralTokenResponse {
    #[serde(rename = "publicToken")]
    public_token: String,
}

/// Response from Dub API endpoints
#[derive(Debug, Deserialize)]
struct DubApiResponse {
    #[serde(rename = "eventId")]
    event_id: Option<String>,
    message: Option<String>,
}

impl DubClient {
    /// Create a new Dub client with the provided configuration
    pub fn new(config: DubConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .user_agent("Narrativ/1.0")
            .build()
            .context("Failed to create HTTP client for Dub API")?;

        Ok(DubClient { client, config })
    }

    /// Track a lead event via Dub API
    #[instrument(skip(self, event), fields(customer_id = %event.customer_id, event_name = %event.event_name))]
    pub async fn track_lead(&self, event: DubLeadEvent) -> Result<DubEventResponse> {
        if !self.config.enabled {
            warn!("Dub tracking is disabled, skipping lead event");
            return Ok(DubEventResponse {
                success: true,
                event_id: None,
                message: Some("Tracking disabled".to_string()),
            });
        }

        let request_payload = DubLeadRequest {
            customer_id: event.customer_id,
            customer_email: event.customer_email,
            event_name: event.event_name,
            click_id: event.click_id,
            metadata: event.metadata,
        };

        let url = format!("{}/track/lead", self.config.base_url);
        
        let response = self
            .build_request(&url)?
            .json(&request_payload)
            .send()
            .await
            .context("Failed to send lead event to Dub API")?;

        self.handle_response(response).await
    }

    /// Track a sale event via Dub API
    #[instrument(skip(self, event), fields(customer_id = %event.customer_id, amount_cents = %event.amount_cents, event_name = %event.event_name))]
    pub async fn track_sale(&self, event: DubSaleEvent) -> Result<DubEventResponse> {
        if !self.config.enabled {
            warn!("Dub tracking is disabled, skipping sale event");
            return Ok(DubEventResponse {
                success: true,
                event_id: None,
                message: Some("Tracking disabled".to_string()),
            });
        }

        // Dub API expects amount in cents (confirmed by documentation)
        let amount_in_cents = event.amount_cents as f64;

        let request_payload = DubSaleRequest {
            customer_id: event.customer_id,
            amount: amount_in_cents,
            currency: event.currency,
            event_name: event.event_name,
            click_id: event.click_id,
            metadata: event.metadata,
        };

        log::info!("[DUB SALE] Sending to Dub API - amount_cents: {}, amount_in_cents: {}, payload: {:?}", event.amount_cents, amount_in_cents, request_payload);

        let url = format!("{}/track/sale", self.config.base_url);
        
        let response = self
            .build_request(&url)?
            .json(&request_payload)
            .send()
            .await
            .context("Failed to send sale event to Dub API")?;

        self.handle_response(response).await
    }

    /// Generate a referral token for embedded referral dashboard
    #[instrument(skip(self), fields(tenant_id = %tenant_id))]
    pub async fn generate_referral_token(
        &self,
        tenant_id: String,
        partner_name: String,
        partner_email: String,
        partner_image: Option<String>,
        group_id: Option<String>,
    ) -> Result<String> {
        if !self.config.enabled {
            warn!("Dub tracking is disabled, cannot generate referral token");
            return Err(anyhow::anyhow!("Dub tracking is disabled"));
        }

        let request_payload = DubReferralTokenRequest {
            tenant_id: tenant_id.clone(),
            partner: DubPartnerInfo {
                name: partner_name,
                email: partner_email,
                image: partner_image,
                tenant_id: tenant_id.clone(),
                group_id,
            },
        };

        let url = format!("{}/tokens/embed/referrals", self.config.base_url);
        
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request_payload)
            .send()
            .await
            .context("Failed to send referral token request to Dub API")?;

        let status = response.status();
        
        if status.is_success() {
            let token_response: DubReferralTokenResponse = response
                .json()
                .await
                .context("Failed to parse referral token response from Dub API")?;

            Ok(token_response.public_token)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            warn!("Dub API returned error status {} for referral token: {}", status, error_text);
            Err(anyhow::anyhow!("Dub API error {}: {}", status, error_text))
        }
    }

    /// Build a properly authenticated request
    fn build_request(&self, url: &str) -> Result<RequestBuilder> {
        let request = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json");

        Ok(request)
    }

    /// Handle the response from Dub API
    #[instrument(skip(self, response))]
    async fn handle_response(&self, response: reqwest::Response) -> Result<DubEventResponse> {
        let status = response.status();
        
        if status.is_success() {
            let api_response: DubApiResponse = response
                .json()
                .await
                .context("Failed to parse successful response from Dub API")?;

            Ok(DubEventResponse {
                success: true,
                event_id: api_response.event_id,
                message: api_response.message,
            })
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            warn!("Dub API returned error status {}: {}", status, error_text);

            Ok(DubEventResponse {
                success: false,
                event_id: None,
                message: Some(format!("API error {}: {}", status, error_text)),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dub_client_creation() {
        let config = DubConfig::test();
        let client = DubClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_dub_client_disabled() {
        let config = DubConfig::disabled();
        let client = DubClient::new(config).unwrap();
        
        // Should not fail when disabled
        assert!(!client.config.enabled);
    }

    #[tokio::test]
    async fn test_track_lead_disabled() {
        let config = DubConfig::disabled();
        let client = DubClient::new(config).unwrap();
        
        let event = DubLeadEvent::new_signup(
            uuid::Uuid::new_v4(),
            "test@example.com".to_string()
        );
        
        let response = client.track_lead(event).await.unwrap();
        assert!(response.success);
        assert!(response.message.unwrap().contains("disabled"));
    }

    #[tokio::test]
    async fn test_track_sale_disabled() {
        let config = DubConfig::disabled();
        let client = DubClient::new(config).unwrap();
        
        let event = DubSaleEvent::new_subscription("test-customer".to_string(), 2999);
        
        let response = client.track_sale(event).await.unwrap();
        assert!(response.success);
        assert!(response.message.unwrap().contains("disabled"));
    }
}
