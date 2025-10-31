//! Counts styles for a user, with filtering.

#[tracing::instrument(skip(pool))]
pub async fn count_styles_for_user(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    search_pattern: &str,
    org_ids: &[uuid::Uuid],
    is_favorite_filter: std::option::Option<bool>,
    is_public_filter: std::option::Option<bool>,
) -> std::result::Result<i64, sqlx::Error> {
    let sql_object_type_style: &str = "style";
    let sql_entity_type_user_enum: crate::db::shares::EntityType = crate::db::shares::EntityType::User;
    let sql_entity_type_org_enum: crate::db::shares::EntityType = crate::db::shares::EntityType::Organization;
    let org_ids_param = if org_ids.is_empty() {
        &[] as &[uuid::Uuid]
    } else {
        org_ids
    };


    let total_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(DISTINCT s.id)
        FROM styles s
        LEFT JOIN object_shares os_user ON s.id = os_user.object_id 
            AND os_user.object_type = $3 
            AND os_user.entity_type = $4 
            AND os_user.entity_id = $1   
        LEFT JOIN object_shares os_org ON s.id = os_org.object_id
            AND os_org.object_type = $3 
            AND os_org.entity_type = $5 
            AND os_org.entity_id = ANY($6) 
        WHERE (
            CASE 
                WHEN $8::BOOLEAN IS NULL THEN
                    -- Default behavior: return both user's styles and public styles
                    (s.user_id = $1 OR s.is_public = true OR os_user.id IS NOT NULL OR os_org.id IS NOT NULL)
                WHEN $8::BOOLEAN = true THEN
                    -- Only public styles regardless of user_id
                    s.is_public = true
                WHEN $8::BOOLEAN = false THEN
                    -- User's styles + organization shared styles, but exclude public styles
                    (s.user_id = $1 OR os_user.id IS NOT NULL OR os_org.id IS NOT NULL) AND s.is_public = false
                ELSE
                    -- Fallback to default behavior
                    (s.user_id = $1 OR s.is_public = true OR os_user.id IS NOT NULL OR os_org.id IS NOT NULL)
            END
        )
        AND s.name ILIKE $2 
        AND ($7::BOOLEAN IS NULL OR EXISTS(SELECT 1 FROM user_favorites WHERE user_id = $1 AND entity_id = s.id AND entity_type = 'style') = $7)
        "#,
        user_id,                // $1
        search_pattern,         // $2
        sql_object_type_style,  // $3
        sql_entity_type_user_enum as _, // $4
        sql_entity_type_org_enum as _,  // $5
        org_ids_param,          // $6
        is_favorite_filter,     // $7
        is_public_filter        // $8
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);
    
    // DEBUG: Log the count result
    log::info!("DEBUG count_styles_for_user result: total_count={}", total_count);

    Ok(total_count)
}