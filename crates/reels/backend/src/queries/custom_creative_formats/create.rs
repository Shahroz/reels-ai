//! Inserts a new custom creative format into the database.
//!
//! This function takes a user ID and format data, creates a new record in the
//! `custom_creative_formats` table, and returns the newly created format.
//! It encapsulates the direct database interaction for creating a custom format.

pub async fn create_custom_creative_format(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    format_data: &crate::routes::formats::create_custom_format_request::CreateCustomFormatRequest,
) -> std::result::Result<crate::db::custom_creative_formats::CustomCreativeFormat, sqlx::Error> {
    let is_public = format_data.is_public.unwrap_or(false);
    // Set user_id to NULL if the format is public, otherwise use the provided user_id
    let effective_user_id = if is_public { None } else { Some(user_id) };

   sqlx::query_as!(
       crate::db::custom_creative_formats::CustomCreativeFormat,
       r#"
        INSERT INTO custom_creative_formats (user_id, name, description, width, height, json_schema, metadata, creative_type, is_public)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, user_id, name, description, width, height, json_schema, metadata, creative_type, created_at, updated_at, is_public
        "#,
        effective_user_id,
        format_data.name,
        format_data.description,
        format_data.width,
        format_data.height,
        format_data.json_schema,
        format_data.metadata,
        format_data.creative_type.to_string(),
        is_public
    )
    .fetch_one(pool)
    .await
}
