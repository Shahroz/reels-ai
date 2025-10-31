//! Configuration for Dub attribution tracking routes
//!
//! This module configures the API routes for Dub lead and sale tracking endpoints.

use actix_web::web;

use super::generate_referral_token;
use super::track_lead;
