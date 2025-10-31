//! Defines the query function for updating a user database collection.
//!
//! This function encapsulates the logic to fetch an existing collection,
//! determine the final values for its name and description based on potentially
//! partial input, and then perform the database update.
//! Adheres to 'one item per file', FQN, and other `rust_guidelines.md`.

/// Updates a user's database collection with new name and/or description.
///
/// Fetches the collection first to ensure it exists and belongs to the user.
/// If `name` is `None`, the current name is preserved.
/// If `description` is `None`, the current description is preserved.
/// If `description` is `Some(None)`, the description is set to NULL.
///
/// # Arguments
/// * `pool` - The database connection pool.
/// * `user_id` - The ID of the user performing the update.
/// * `collection_id_to_update` - The ID of the collection to be updated.
/// * `new_name` - An `Option<String>` for the new collection name.
/// * `new_description` - An `Option<Option<String>>` for the new collection description.
///
/// # Returns
/// A `Result` containing the updated `UserDbCollection` on success,
/// or an `sqlx::Error` on failure (e.g., `RowNotFound` if the collection
/// doesn't exist or doesn't belong to the user).
pub async fn update_user_db_collection_query(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    collection_id_to_update: uuid::Uuid,
    new_name: Option<String>,
    new_description: Option<Option<String>>,
) -> Result<crate::db::user_db_collection::UserDbCollection, sqlx::Error> {
    // Fetch current collection to ensure it exists and is owned by user,
    // and to use its current values if some fields in request are None.
    let current_collection = match sqlx::query_as!(
        crate::db::user_db_collection::UserDbCollection,
        r#"SELECT 
            id AS "id: uuid::Uuid",
            user_id AS "user_id: uuid::Uuid",
            name,
            description,
            schema_definition AS "schema_definition: serde_json::Value",
            source_predefined_collection_id AS "source_predefined_collection_id: uuid::Uuid",
            ui_component_definition AS "ui_component_definition: serde_json::Value",
            created_at,
            updated_at
           FROM user_db_collections WHERE id = $1 AND user_id = $2"#,
        collection_id_to_update,
        user_id
    )
    .fetch_one(pool) // Use fetch_one to return RowNotFound if not found/not owned
    .await {
        Ok(coll) => coll,
        Err(e) => return std::result::Result::Err(e), // Propagate error (e.g., RowNotFound)
    };

    let name_to_set = new_name.unwrap_or(current_collection.name);
    let description_to_set = match new_description {
        Some(desc_opt) => desc_opt, // This can be Some(String) or None (to clear description)
        None => current_collection.description, // Preserve current description
    };

    sqlx::query_as!(
        crate::db::user_db_collection::UserDbCollection,
        r#"
        UPDATE user_db_collections
        SET name = $1, description = $2, updated_at = NOW()
        WHERE id = $3 AND user_id = $4
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
        name_to_set,
        description_to_set,
        collection_id_to_update,
        user_id
    )
    .fetch_one(pool)
    .await
}
