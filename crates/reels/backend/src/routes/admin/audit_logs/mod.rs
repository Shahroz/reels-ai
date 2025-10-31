//! Admin audit logs routes and related types.
//!
//! This module contains all handlers, request/response types, and configuration
//! for the audit logs admin endpoints. Audit logs provide a complete trail of
//! administrative actions for security, compliance, and debugging purposes.

pub mod configure_audit_logs_routes;
pub mod list_audit_logs_handler;
pub mod list_audit_logs_params;
pub mod list_audit_logs_response;
pub mod get_audit_log_handler;

