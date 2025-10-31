//! Defines a struct for documenting multipart file uploads in OpenAPI.

use utoipa::ToSchema;

/// Represents a file to be uploaded in a multipart/form-data request.
#[derive(ToSchema)]
pub struct FileUpload {
    /// The file to upload.
    #[schema(format = "binary")]
    pub file: String,
} 