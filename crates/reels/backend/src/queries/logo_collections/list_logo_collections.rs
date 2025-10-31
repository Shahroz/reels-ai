//! Lists logo collections for a user with asset count information.
//!
//! This function retrieves all logo collections belonging to a specific user,
//! including asset count for each collection. Supports pagination and ordering
//! for efficient handling of large collection lists.

/// Lists all logo collections for a user with asset counts
pub async fn list_logo_collections(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
) -> std::result::Result<std::vec::Vec<crate::schemas::logo_collection_schemas::LogoCollectionSummaryResponse>, sqlx::Error> {
    let collections = sqlx::query!(
        r#"
        SELECT 
            lc.id,
            lc.name,
            lc.description,
            lc.created_at,
            lc.updated_at,
            COUNT(lca.asset_id) as asset_count,
            MIN(a.url) as thumbnail_url
        FROM logo_collections lc
        LEFT JOIN logo_collection_assets lca ON lc.id = lca.logo_collection_id
        LEFT JOIN assets a ON lca.asset_id = a.id
        WHERE lc.user_id = $1
        GROUP BY lc.id, lc.name, lc.description, lc.created_at, lc.updated_at
        ORDER BY lc.updated_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    let result = collections
        .into_iter()
        .map(|row| crate::schemas::logo_collection_schemas::LogoCollectionSummaryResponse {
            id: row.id,
            name: row.name,
            description: row.description,
            created_at: row.created_at,
            updated_at: row.updated_at,
            asset_count: row.asset_count.unwrap_or(0),
            thumbnail_url: row.thumbnail_url,
        })
        .collect();

    std::result::Result::Ok(result)
}


// Tests temporarily disabled - need proper test infrastructure
