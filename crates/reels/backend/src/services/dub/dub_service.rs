//! Dub service implementation for lead attribution tracking
//!
//! This service provides the main implementation of DubServiceTrait,
//! handling lead and sale event tracking with proper error handling and observability.

use anyhow::Result;
use async_trait::async_trait;
use tracing::{error, info, instrument, warn};


use super::dub_client::DubClient;
use super::dub_config::DubConfig;
use super::dub_service_trait::{DubEventResponse, DubLeadEvent, DubSaleEvent, DubServiceTrait};

/// Main Dub service implementation
#[derive(Debug, Clone)]
pub struct DubService {
    client: DubClient,
    config: DubConfig,
}

impl DubService {
    /// Create a new Dub service from configuration
    pub fn new(config: DubConfig) -> Result<Self> {
        let client = DubClient::new(config.clone())?;
        
        if config.enabled {
            info!("Dub service initialized with workspace: {}", config.workspace_id);
        } else {
            info!("Dub service initialized but tracking is disabled");
        }

        Ok(DubService { client, config })
    }

    /// Create a new Dub service from environment variables
    pub fn from_env() -> Result<Self> {
        let config = DubConfig::from_env()?;
        Self::new(config)
    }

    /// Create a disabled Dub service for testing
    #[cfg(test)]
    pub fn disabled() -> Result<Self> {
        let config = DubConfig::disabled();
        Self::new(config)
    }
}

#[async_trait]
impl DubServiceTrait for DubService {
    #[instrument(skip(self, event), fields(customer_id = %event.customer_id, event_name = %event.event_name))]
    async fn track_lead_event(&self, event: DubLeadEvent) -> Result<DubEventResponse> {
        if !self.config.enabled {
            warn!("Dub tracking disabled, skipping lead event for customer: {}", event.customer_id);
            return Ok(DubEventResponse {
                success: true,
                event_id: None,
                message: Some("Tracking disabled".to_string()),
            });
        }

        // Skip API call if no click_id (attribution) is available
        // Dub API requires clickId, so we can't track non-attributed events
        if event.click_id.is_none() {
            info!("Skipping Dub lead tracking for customer {} - no attribution data (click_id)", event.customer_id);
            return Ok(DubEventResponse {
                success: true,
                event_id: None,
                message: Some("No attribution data available".to_string()),
            });
        }

        info!("Tracking lead event: {} for customer: {} with attribution", event.event_name, event.customer_id);

        match self.client.track_lead(event.clone()).await {
            Ok(response) => {
                if response.success {
                    info!(
                        "Successfully tracked lead event for customer: {} with event_id: {:?}",
                        event.customer_id, response.event_id
                    );
                } else {
                    warn!(
                        "Dub API reported failure for lead event (customer: {}): {:?}",
                        event.customer_id, response.message
                    );
                }
                Ok(response)
            }
            Err(e) => {
                error!(
                    "Failed to track lead event for customer: {} - {}",
                    event.customer_id, e
                );
                // Return error but don't fail the calling operation
                Ok(DubEventResponse {
                    success: false,
                    event_id: None,
                    message: Some(format!("Client error: {}", e)),
                })
            }
        }
    }

    #[instrument(skip(self, event), fields(customer_id = %event.customer_id, amount_cents = %event.amount_cents, event_name = %event.event_name))]
    async fn track_sale_event(&self, event: DubSaleEvent) -> Result<DubEventResponse> {
        if !self.config.enabled {
            warn!("Dub tracking disabled, skipping sale event for customer: {}", event.customer_id);
            return Ok(DubEventResponse {
                success: true,
                event_id: None,
                message: Some("Tracking disabled".to_string()),
            });
        }

        info!(
            "Tracking sale event: {} for customer: {} (amount: ${:.2})",
            event.event_name,
            event.customer_id,
            event.amount_cents as f64 / 100.0
        );

        match self.client.track_sale(event.clone()).await {
            Ok(response) => {
                if response.success {
                    info!(
                        "Successfully tracked sale event for customer: {} with event_id: {:?} (amount: ${:.2})",
                        event.customer_id,
                        response.event_id,
                        event.amount_cents as f64 / 100.0
                    );
                } else {
                    warn!(
                        "Dub API reported failure for sale event (customer: {}): {:?}",
                        event.customer_id, response.message
                    );
                }
                Ok(response)
            }
            Err(e) => {
                error!(
                    "Failed to track sale event for customer: {} - {}",
                    event.customer_id, e
                );
                // Return error but don't fail the calling operation
                Ok(DubEventResponse {
                    success: false,
                    event_id: None,
                    message: Some(format!("Client error: {}", e)),
                })
            }
        }
    }

    #[instrument(skip(self), fields(tenant_id = %tenant_id))]
    async fn generate_referral_token(
        &self,
        tenant_id: String,
        partner_name: String,
        partner_email: String,
        partner_image: Option<String>,
        group_id: Option<String>,
    ) -> Result<String> {
        if !self.config.enabled {
            warn!("Dub tracking disabled, cannot generate referral token for tenant: {}", tenant_id);
            return Err(anyhow::anyhow!("Dub tracking is disabled"));
        }

        info!("Generating referral token for tenant: {} (partner: {})", tenant_id, partner_name);

        match self.client.generate_referral_token(
            tenant_id.clone(),
            partner_name.clone(),
            partner_email.clone(),
            partner_image.clone(),
            group_id.clone(),
        ).await {
            Ok(token) => {
                info!("Successfully generated referral token for tenant: {}", tenant_id);
                Ok(token)
            }
            Err(e) => {
                error!("Failed to generate referral token for tenant: {} - {}", tenant_id, e);
                Err(e)
            }
        }
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    fn get_workspace_id(&self) -> Option<String> {
        if self.config.enabled && !self.config.workspace_id.is_empty() {
            Some(self.config.workspace_id.clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_dub_service_creation() {
        let service = DubService::disabled().unwrap();
        assert!(!service.is_enabled());
        assert!(service.get_workspace_id().is_none());
    }

    #[tokio::test]
    async fn test_track_lead_event_disabled() {
        let service = DubService::disabled().unwrap();
        let event = DubLeadEvent::new_signup(Uuid::new_v4(), "test@example.com".to_string());
        
        let response = service.track_lead_event(event).await.unwrap();
        assert!(response.success);
        assert!(response.message.unwrap().contains("disabled"));
    }

    #[tokio::test]
    async fn test_track_sale_event_disabled() {
        let service = DubService::disabled().unwrap();
        let event = DubSaleEvent::new_subscription("test-customer".to_string(), 2999);
        
        let response = service.track_sale_event(event).await.unwrap();
        assert!(response.success);
        assert!(response.message.unwrap().contains("disabled"));
    }

    #[test]
    fn test_workspace_id_when_enabled() {
        let mut config = DubConfig::test();
        config.enabled = true;
        config.workspace_id = "test-workspace-123".to_string();
        
        let service = DubService::new(config).unwrap();
        assert!(service.is_enabled());
        assert_eq!(service.get_workspace_id(), Some("test-workspace-123".to_string()));
    }

    #[test]
    fn test_workspace_id_when_disabled() {
        let service = DubService::disabled().unwrap();
        assert!(!service.is_enabled());
        assert!(service.get_workspace_id().is_none());
    }
}
