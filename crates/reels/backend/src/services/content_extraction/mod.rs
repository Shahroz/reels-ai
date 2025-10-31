//! A service for extracting text content from various file types.
//!
//! This module provides functionalities to process file blobs (e.g., from PDFs, DOCX)
//! and extract the textual content using multimodal LLMs.
 
pub mod extract_text;
pub mod should_use_file_api;
pub mod extract_text_with_file_api; 