//! Property contents route module.
//!
//! This module handles the generation of marketing content from property descriptions
//! using the GenNodes workflow. It processes video tour documents to extract property
//! descriptions and generates multiple marketing documents for different platforms.
//! Generated documents inherit collection_id from source documents for proper listing organization.

pub mod create_property_contents;
pub mod create_property_contents_with_studio_journey;
pub mod parse_property_marketing_response;
pub mod property_content_templates;
pub mod configure_property_contents_routes; 