//! Defines the request body structure for updating the schema of a user DB collection.
//!
//! This module contains the primary request struct `UpdateUserDbCollectionSchemaRequest`
//! and the `UpdateUserDbCollectionSchemaPayload` enum it uses. This allows updates
//! to be specified either directly with a new schema or via natural language instruction.
//! Adheres to 'one item per file' (conceptually, for the request) and FQN guidelines.

/// Represents the different ways a user DB collection schema update can be specified.
///
/// Used within `UpdateUserDbCollectionSchemaRequest` to determine if the update
/// is a direct schema replacement or an instruction-based modification.
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")] // Ensures JSON representation is camelCase for variants
pub enum UpdateUserDbCollectionSchemaPayload {
    /// Specifies the new schema directly.
    Direct {
        /// The complete new JSON schema definition for the collection.
        #[schema(
            value_type = Object, // Hints to OpenAPI that this is a flexible JSON object
            example = json!({"type": "object", "properties": {"new_field": {"type": "string", "description": "A new text field"}}, "required": ["new_field"]})
        )]
        schema_definition: serde_json::Value,
    },
    /// Specifies changes to the schema via a natural language instruction.
    InstructionBased {
        /// A natural language instruction detailing the desired modifications
        /// to the existing schema. The LLM will interpret this to generate the new schema.
        #[schema(example = "Add an 'email' field of type string, mark 'name' as required, and remove the 'old_field'.")]
        instruction: std::string::String,
    },
}

/// The request body for the endpoint that updates a user DB collection's schema.
///
/// This struct encapsulates the payload, which can be one of the variants defined
/// in `UpdateUserDbCollectionSchemaPayload`. The `utoipa::ToSchema` derive, combined
/// with `#[serde(flatten)]` on the payload, should guide OpenAPI generation
/// to use `oneOf` for the payload variants, as per common patterns and `zenide.md` expectations.
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateUserDbCollectionSchemaRequest {
    /// The payload defining how the schema update should be performed.
    /// `#[serde(flatten)]` is used to inline the enum variants into the request object structure
    /// for the JSON representation, which helps `utoipa` generate a `oneOf` schema.
    #[serde(flatten)]
    #[schema(value_type = UpdateUserDbCollectionSchemaPayload)] // Tells utoipa to use the schema of UpdateUserDbCollectionSchemaPayload
    pub payload: UpdateUserDbCollectionSchemaPayload,
}
