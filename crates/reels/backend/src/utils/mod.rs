//! Utility functions shared across the backend.
//!
//! This module houses common helper functions and processors
//! to avoid code duplication and promote modularity.
//! Each utility should reside in its own file following the guidelines.

pub mod extract_html_colors;
pub mod html_minimizer;

pub mod color_conversions;
pub mod jwt;
pub mod sanitize_llm_html_output;
pub mod minimize_large_html_content;

pub mod string_patcher;
pub mod password_validator;
