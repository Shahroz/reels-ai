//! Provides structures and functions for interacting with the OpenAI Chat Completion API.
//! 
//! This module organizes the various data types (like messages, roles, models)
//! and API call functions required for making requests to OpenAI.
//! It follows the one-item-per-file structure mandated by project guidelines.
//! All items must be accessed via their respective submodules (e.g., `crate::llm::vendors::openai::role::Role`).

pub mod call_gpt;
pub mod call_gpt_with_body;
pub mod message;
pub mod openai_chat_completion_request;
pub mod openai_model;
pub mod reasoning;
pub mod reasoning_effort;
pub mod response_format;
pub mod response_type;
pub mod role;
pub mod string_or_array;
pub mod tool;
pub mod tool_choice;
pub mod tool_type;
pub mod sora_video_request;

// pub mod stream_chat_completion;
// Note: No `pub use` statements are included to strictly adhere to the guideline
// of using fully qualified paths, even for items within this module's submodules.
// Consumers will use paths like `crate::llm::vendors::openai::message::Message`.
