//! Inserts a new custom creative format as a copy of an existing one.
//!
//! This function takes a fully populated `CustomCreativeFormat` struct
//! and inserts it into the `custom_creative_formats` table.
//! It's used by the copy operation to save the duplicated format.
//! Returns the newly created format record from the database.

pub async fn insert_copy(
    pool: &sqlx::PgPool,
    new_format: &crate::db::custom_creative_formats::CustomCreativeFormat,
) -> std::result::Result<crate::db::custom_creative_formats::CustomCreativeFormat, sqlx::Error> {
    sqlx::query_as!(
        crate::db::custom_creative_formats::CustomCreativeFormat,
        r#"
        INSERT INTO custom_creative_formats
            (id, user_id, name, description, width, height, creative_type, json_schema, is_public, metadata, created_at, updated_at)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id, user_id, name, description, width, height, creative_type, json_schema, is_public, metadata, created_at, updated_at
        "#,
        new_format.id,
        new_format.user_id,
        new_format.name,
        new_format.description,
        new_format.width,
        new_format.height,
        new_format.creative_type,
        new_format.json_schema,
        new_format.is_public,
        new_format.metadata,
        new_format.created_at,
        new_format.updated_at
    )
    .fetch_one(pool)
    .await
}

#[cfg(test)]
mod tests {
    // Integration tests would be needed to test this database interaction.
}