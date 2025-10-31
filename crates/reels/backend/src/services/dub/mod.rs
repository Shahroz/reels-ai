//! Dub lead attribution tracking service
//!
//! This module provides integration with Dub's attribution tracking API,
//! enabling lead and sale event tracking for marketing attribution.

pub mod dub_client;
pub mod dub_config;
pub mod dub_service;
pub mod dub_service_trait;

#[cfg(test)]
pub mod dub_service_test;

// Re-export commonly used types
pub use dub_config::DubConfig;
pub use dub_service::DubService;
pub use dub_service_trait::{
    DubEventResponse, DubLeadEvent, DubSaleEvent, DubServiceTrait,
};
