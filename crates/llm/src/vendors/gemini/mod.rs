pub mod gemini_model;
pub mod completion;
pub mod gemini_request;
pub mod content;
pub mod part;
pub mod generation_config;
pub mod safety_setting;
pub mod system_instruction;
pub mod tool;
pub mod api_response;
pub mod candidate;
pub mod content_response;
pub mod completion_conversation;
pub mod part_response;
pub mod function_call_response;
pub mod gemini_output;
pub mod system_instruction_text_payload;
pub mod inline_data; // Moved to maintain alphabetical-like order after deletions
pub mod file_data; // File API support for large files
pub mod file_api_client; // File API client for uploads and management
pub mod file_info; // File information from File API
pub mod file_upload_response; // Response structure for file uploads
pub mod google_search; // Moved to maintain alphabetical-like order after deletions
pub mod role;
pub mod function_declaration;
pub mod function_parameters_schema;
pub mod function_result_part;
pub mod property_definition;
pub mod video_metadata;
pub mod veo3_video_request;
pub mod veo3_video_generation;
pub mod gcp_auth;
// Video metadata for uploaded video files

// Re-exports
pub use file_info::FileInfo;
pub use file_upload_response::FileUploadResponse;
pub use video_metadata::VideoMetadata;
