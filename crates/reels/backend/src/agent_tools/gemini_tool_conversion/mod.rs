//! Module for converting Narrativ backend tools to Gemini API compatible format.
//!
//! This module provides structures and functions to transform internal tool
//! definitions, including their parameter schemas, into the format expected
//! by Google's Gemini API for function calling.
//! Adheres strictly to project Rust coding standards.

pub mod convert_narrativ_tools_to_gemini_format;

// No re-exports are made here to enforce usage of fully qualified paths
// as per project guidelines. For example, to use GeminiTools:
// crate::llm::src::vendors::gemini::gemini_function_tool::GeminiFunctionTool (formerly GeminiTools)
