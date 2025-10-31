//! Module for color conversion utilities.
//!
//! This module provides functions to convert various CSS color formats (named, RGB, HSL)
//! into a standardized hexadecimal representation. Each submodule handles a specific
//! type of conversion or parsing.
//! Adheres to one-item-per-file, no_use_statements, and other rust_guidelines.

pub mod named_to_hex;
pub mod rgb_string_to_hex;
pub mod hsl_string_to_hex;