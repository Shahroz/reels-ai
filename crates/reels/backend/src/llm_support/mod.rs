//! Module for LLM support structures, like wrappers for typed LLM interactions.
//!
//! This module provides helper types and implementations required for
//! interacting with the `llm_typed` functionality, especially when dealing
//! with complex or generic types like `serde_json::Value` as LLM outputs.
//! Adheres to `rust_guidelines.md`.

pub mod json_schema_container;
