//! Updates an existing custom creative format in the database.
//!
//! This function updates the specified fields for a custom creative format.
//! It ensures that the format exists and is owned by the specified user
//! by including `id = $1` and `user_id = $2` in the `WHERE` clause.
//! If no row is found, it returns a `sqlx::Error::RowNotFound`.
//! Admin users can update any format regardless of ownership.

pub async fn update(
    pool: &sqlx::PgPool,
    format_id: uuid::Uuid,
    user_id: uuid::Uuid,
    is_admin: bool,
    format_data: &crate::routes::formats::create_custom_format_request::CreateCustomFormatRequest,
) -> std::result::Result<crate::db::custom_creative_formats::CustomCreativeFormat, sqlx::Error> {
    let is_public = format_data.is_public.unwrap_or(false);
    // Set user_id to NULL if the format is public, otherwise use the provided user_id
    let effective_user_id = if is_public { None } else { Some(user_id) };

    // If user is admin, they can update any format, so we only check for format existence
    // If user is not admin, they can only update their own formats
    if is_admin {
        sqlx::query_as!(
            crate::db::custom_creative_formats::CustomCreativeFormat,
            r#"
            UPDATE custom_creative_formats
            SET name = $2, description = $3, width = $4, height = $5, json_schema = $6, metadata = $7, is_public = $8, user_id = $9, updated_at = now()
            WHERE id = $1
            RETURNING id, user_id, name, description, width, height, json_schema, metadata, created_at, updated_at, creative_type, is_public
            "#,
            format_id,
            format_data.name.clone(),
            format_data.description.clone(),
            format_data.width,
            format_data.height,
            format_data.json_schema.clone(),
            format_data.metadata.clone(),
            is_public,
            effective_user_id
        )
        .fetch_one(pool)
        .await
    } else {
        sqlx::query_as!(
            crate::db::custom_creative_formats::CustomCreativeFormat,
            r#"
            UPDATE custom_creative_formats
            SET name = $3, description = $4, width = $5, height = $6, json_schema = $7, metadata = $8, is_public = $9, user_id = $10, updated_at = now()
            WHERE id = $1 AND user_id = $2
            RETURNING id, user_id, name, description, width, height, json_schema, metadata, created_at, updated_at, creative_type, is_public
            "#,
            format_id,
            user_id,
            format_data.name.clone(),
            format_data.description.clone(),
            format_data.width,
            format_data.height,
            format_data.json_schema.clone(),
            format_data.metadata.clone(),
            is_public,
            effective_user_id
        )
        .fetch_one(pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    // Integration tests would be needed to test this database interaction.
}