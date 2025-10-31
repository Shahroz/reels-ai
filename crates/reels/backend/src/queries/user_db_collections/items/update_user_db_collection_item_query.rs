
//! Provides a query to merge data into an existing item in a user DB collection.
//!
//! This function implements a merge/patch operation. It verifies collection ownership,
//! fetches the existing item, merges the provided JSON patch, validates the
//! result against the collection's schema, and performs the database update.
//! It adheres to 'one item per file' and FQN guidelines.

// Note: `anyhow::Error` is used for error handling. Specific error types
// could be defined for more granular error management if a project-wide
// error handling strategy (e.g., using `thiserror`) is in place.

pub async fn update_user_db_collection_item_query(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    collection_id_uuid: uuid::Uuid,
    item_id_uuid: uuid::Uuid,
    item_data_patch: serde_json::Value,
) -> Result<crate::db::user_db_collection_item::UserDbCollectionItem, anyhow::Error> {
    // 1. Verify ownership of parent collection and get schema_definition
    let schema_definition_json = match sqlx::query!(
        r#"
        SELECT user_id, schema_definition
        FROM user_db_collections
        WHERE id = $1
        "#,
        collection_id_uuid
    )
   .fetch_optional(pool)
   .await? // Propagate sqlx::Error via anyhow::Error
   {
       Some(record) => {
            if record.user_id != user_id {
                anyhow::bail!("UserDoesNotOwnCollection: User does not own the parent collection.");
            }
            record.schema_definition
        }
        None => {
            anyhow::bail!("CollectionNotFound: Parent collection not found.");
        }
   };

    // 2. Fetch the existing item data
    let mut current_item_data = match sqlx::query!(
        r#"SELECT item_data FROM user_db_collection_items WHERE id = $1 AND user_db_collection_id = $2"#,
        item_id_uuid,
        collection_id_uuid
    )
    .fetch_optional(pool)
    .await?
    {
        Some(record) => record.item_data,
        None => anyhow::bail!("ItemNotFound: Item not found in the specified collection."),
    };

    // 3. Merge patch into current data
    if let (serde_json::Value::Object(target), serde_json::Value::Object(patch)) =
        (&mut current_item_data, &item_data_patch)
    {
        for (key, value) in patch {
            target.insert(key.clone(), value.clone());
        }
    } else {
        anyhow::bail!("InvalidPatch: The provided data patch must be a JSON object.");
    }

    // 4. Compile the schema and validate the merged item data
    let compiled_schema = match jsonschema::Validator::new(&schema_definition_json) {
        Ok(s) => s,
        Err(e) => {
            anyhow::bail!("Schema compilation error: {}", e);
        }
    };

    if let Err(validation_errors) = compiled_schema.validate(&current_item_data) {
        let error_messages: std::vec::Vec<std::string::String> = vec![validation_errors.to_string()];
        let error_string = error_messages.join(", ");
        anyhow::bail!(
            "ValidationFailed: Item data does not conform to schema: {}",
            error_string
        );
    }

    // 5. Update item with merged data
    match sqlx::query_as!(
        crate::db::user_db_collection_item::UserDbCollectionItem,
        r#"
        UPDATE user_db_collection_items
        SET item_data = $1, updated_at = NOW()
        WHERE id = $2 AND user_db_collection_id = $3
        RETURNING id, user_db_collection_id, item_data, created_at, updated_at
        "#,
        current_item_data,
        item_id_uuid,
        collection_id_uuid
    )
    .fetch_one(pool)
    .await
    {
        Ok(item) => std::result::Result::Ok(item),
        Err(sqlx::Error::RowNotFound) => {
            anyhow::bail!("ItemNotFound: Item not found in the specified collection or update failed.");
        }
        Err(e) => {
            log::error!("Failed to update user DB collection item in query: {e:?}");
            anyhow::bail!("DatabaseError: Failed to update collection item. Error: {}", e);
        }
    }
}
