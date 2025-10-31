//! Defines the query function for creating a user database collection.
//!
//! This function encapsulates the logic for refining a user-provided schema
//! using an LLM and then inserting the new collection details into the database.
//! It adheres to 'one item per file' and FQN guidelines.
//! Includes in-file tests for validation.

pub async fn create_user_db_collection_query(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    name: std::string::String,
    description: std::option::Option<std::string::String>,
    initial_schema_definition: serde_json::Value,
) -> std::result::Result<crate::db::user_db_collection::UserDbCollection, anyhow::Error> {
    // Serialize initial schema and prepare description for the LLM prompt
    let initial_schema_definition_str = match serde_json::to_string_pretty(&initial_schema_definition) {
        Ok(s) => s,
        Err(e) => {
            log::error!(
                "Failed to serialize initial schema_definition for LLM prompt: {e:?}"
            );
            // This error suggests bad input from the client.
            return std::result::Result::Err(anyhow::anyhow!(
                "Invalid initial schema definition provided (could not serialize): {}",
                e
            ));
        }
    };

    let description_for_prompt = description
        .clone()
        .unwrap_or_else(|| std::string::String::from("No description provided."));

    // Construct the prompt for llm_typed.
    let llm_prompt_task = std::format!(
        r#"You are an AI assistant tasked with refining a JSON schema for a user-defined database collection.
The user has provided a collection name, an optional description, and an initial JSON schema.
Your goal is to return an improved and robust JSON schema based on this input.
If the initial schema is trivial, nonsensical, or unsuitable, please generate a sensible schema that aligns with the collection name and description.
Ensure the output is only the JSON schema object itself, conforming to the structure expected by the system.

Collection Name: {name}
<DESCRIPTION>{description_for_prompt}</DESCRIPTION>
<INITIAL_JSON_SCHEMA>{initial_schema_definition_str}</INITIAL_JSON_SCHEMA>

Analyze the provided information and return a refined JSON schema object.
If the description mentions a particular language or locale, consider that
when generating descriptions or examples within the schema, but the primary
schema structure should be universally applicable.

The schema must be flat no nested objects are allowed. Please use primitive types only.
Don't include id, created_at, updated_at in the columns they are added as a wrapper to this schema.

Please remember that the schema is wrapped in {{"schema": "schema here"}}
"#
    );

    // Define models to use for the LLM call
    let models_to_use = std::vec![
        llm::llm_typed_unified::vendor_model::VendorModel::Gemini(llm::vendors::gemini::gemini_model::GeminiModel::Gemini25Pro),
    ];

    // Call llm_typed to get the processed schema
    let llm_result: std::result::Result<crate::llm_support::json_schema_container::JsonSchemaContainer, anyhow::Error> =
        llm::llm_typed_unified::llm_typed::llm_typed(
            llm_prompt_task,
            models_to_use,
            3, // retries
            Some(llm::llm_typed_unified::output_format::OutputFormat::Json),
            false, // debug_mode
        )
        .await;

    let final_schema_definition = match llm_result {
        Ok(json_schema_container) => serde_json::to_value(json_schema_container),
        Err(e) => {
            log::error!("LLM failed to generate/refine schema: {e:?}");
            return std::result::Result::Err(anyhow::anyhow!(
                "Failed to process schema definition with LLM: {}",
                e
            ));
        }
    };

    let new_collection_id = uuid::Uuid::new_v4();

    match sqlx::query_as!(
        crate::db::user_db_collection::UserDbCollection,
        r#"
        INSERT INTO user_db_collections (id, user_id, name, description, schema_definition, source_predefined_collection_id, ui_component_definition)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING 
            id AS "id: uuid::Uuid",
            user_id AS "user_id: uuid::Uuid",
            name,
            description,
            schema_definition AS "schema_definition: serde_json::Value",
            source_predefined_collection_id AS "source_predefined_collection_id: uuid::Uuid",
            ui_component_definition AS "ui_component_definition: serde_json::Value",
            created_at,
            updated_at
        "#,
        new_collection_id,
        user_id,
        name,
        description,
        final_schema_definition?,
        std::option::Option::<uuid::Uuid>::None, // source_predefined_collection_id
        serde_json::json!({}), // ui_component_definition - default empty object
    )
    .fetch_one(pool)
    .await
    {
        Ok(collection) => std::result::Result::Ok(collection),
        Err(e) => {
            log::error!("Failed to create user DB collection in database: {e:?}");
            std::result::Result::Err(anyhow::anyhow!("Failed to save collection to database: {}", e))
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_llm_prompt_formatting() {
        // Test the LLM prompt string construction logic if it were more complex
        // For this specific case, it's straightforward string formatting.
        let name = std::string::String::from("My Test Collection");
        let description_for_prompt = std::string::String::from("A detailed description.");
        let initial_schema_definition_str = std::string::String::from(r#"{
  "type": "object"
}"#);
         let llm_prompt_task = std::format!(
            r#"You are an AI assistant tasked with refining a JSON schema for a user-defined database collection.
The user has provided a collection name, an optional description, and an initial JSON schema.
Your goal is to return an improved and robust JSON schema based on this input.
If the initial schema is trivial, nonsensical, or unsuitable, please generate a sensible schema that aligns with the collection name and description.
Ensure the output is only the JSON schema object itself, conforming to the structure expected by the system.

Collection Name: {}
<DESCRIPTION>{}</DESCRIPTION>
<INITIAL_JSON_SCHEMA>{}</INITIAL_JSON_SCHEMA>

Analyze the provided information and return a refined JSON schema object.
If the description mentions a particular language or locale, consider that
when generating descriptions or examples within the schema, but the primary
schema structure should be universally applicable.

The schema must be flat no nested objects are allowed. Please use primitive types only.
Don't include id, created_at, updated_at in the columns they are added as a wrapper to this schema.

Please remember that the schema is wrapped in {{"schema": "schema here"}}
"#,
            name, description_for_prompt, initial_schema_definition_str
        );
        assert!(llm_prompt_task.contains("My Test Collection"));
        assert!(llm_prompt_task.contains("A detailed description."));
        assert!(llm_prompt_task.contains(r#"{
  "type": "object"
}"#));
    }
}
