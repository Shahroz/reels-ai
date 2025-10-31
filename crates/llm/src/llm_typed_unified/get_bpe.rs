//! Provides a helper function to get a BPE (Byte Pair Encoding) instance.
//!
//! This function encapsulates the logic for loading the `cl100k_base` tokenizer
//! from the `tiktoken_rs` crate. It handles potential errors during loading
//! and returns a `Result` containing the `CoreBPE` instance or an error.
//! This utility is used for token counting in LLM interactions.

/// Retrieves an instance of the `CoreBPE` tokenizer.
///
/// This function attempts to load the `cl100k_base` tokenizer.
/// If successful, it returns `Ok(CoreBPE)`.
/// If loading fails, it returns an `Err` with a descriptive message.
pub(crate) fn get_bpe() -> anyhow::Result<tiktoken_rs::CoreBPE> {
    tiktoken_rs::cl100k_base().map_err(|e| anyhow::anyhow!("Failed to load tiktoken BPE: {}", e))
}