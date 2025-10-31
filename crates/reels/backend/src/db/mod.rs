//! Central module for database interaction logic.
//!
//! This module organizes sub-modules responsible for CRUD operations
//! and other database queries related to different entities within the application.
//! It provides a unified access point to various data models.

pub mod api_keys;
pub mod requests;
pub mod research_workflows;
pub mod password_resets;
pub mod verifications;
pub mod users;
pub mod user_status;
pub mod create_oauth_user;
pub mod billing;
pub mod user_subscription;
pub mod user_credit_allocation;
pub mod credit_transaction;
pub mod payment_completions;
pub mod find_or_create_google_user;
pub mod organizations;
pub mod organization_members;
pub mod organization_subscription;
pub mod organization_credit_allocation;
pub mod unlimited_access_grant;
pub mod assets;
pub mod collections;
pub mod creative_type;
pub mod user_db_collection;
pub mod user_db_collection_item;
pub mod predefined_collection;
pub mod bundles;
pub mod creatives;
pub mod pending_invitations;
pub mod documents;
pub mod custom_creative_formats;
pub mod research_conversation;
pub mod styles;
pub mod webflow_creatives;
pub mod create_pool;
pub mod shares; // Added for the new object sharing module
pub mod document_research_usage;
pub mod infinite_research;
pub mod infinite_research_execution;
pub mod infinite_research_list_item;
pub mod one_time_research;
pub mod favorites; // Added for the new user favorites module
pub mod user_google_auth;
pub mod vocal_tours;
#[cfg(feature = "events")]
pub mod analytics_events;
pub mod logo_collection;
pub mod logo_collection_asset;
pub mod watermarking_job;
pub mod studio_journey_shares;
pub mod feed_posts; // Feed functionality - main post table
pub mod feed_post_assets; // Feed functionality - assets within posts
pub mod favorited_prompts; // User favorite enhancement prompts
pub mod audit_logs;
pub mod audit_action;
pub mod credit_rewards;
