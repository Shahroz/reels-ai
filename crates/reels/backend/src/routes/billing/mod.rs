//! Exports all handlers and configurations for the billing feature.
//!
//! This module follows the one-item-per-file pattern. Each file contains a distinct
//! piece of functionality, such as a route handler, a request/response struct, or
//! the route configuration logic.

pub mod configure_billing_routes;
pub mod checkout;
pub mod portal;
pub mod products;
pub mod status;
pub mod plans; 
pub mod access_source;
pub mod billing_status_response;
pub mod payment_status_response;
pub mod get_billing_status_handler;
pub mod get_payment_status_handler; 
