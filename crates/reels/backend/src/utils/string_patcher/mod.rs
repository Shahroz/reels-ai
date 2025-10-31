//! A module for patching strings using patch definitions.
//!
//! This module provides utilities to apply patch operations to in-memory strings,
//! leveraging the `apply-patch` crate's parsing and diffing logic.
//! It's designed for scenarios where file system operations are not desired,
//! using temporary files internally to bridge with the file-based `apply-patch` API.

// pub mod apply_string_patch;
