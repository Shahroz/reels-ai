//! Admin-only database query operations.
//!
//! This module contains queries specifically for administrative operations that require
//! elevated privileges. These queries power admin-only API endpoints for managing users
//! and organizations across the entire platform.

pub mod admin_service_error;
pub mod organizations;
pub mod users;
