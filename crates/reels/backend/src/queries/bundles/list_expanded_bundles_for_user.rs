//! Defines the `list_expanded_bundles_for_user` database query function.
//!
//! This function retrieves all bundles for a specific user ID, expanding
//! associated style, documents, and assets into their full structures.
//! Returns a `Vec<ExpandedBundle>`.
//! Adheres to the project's Rust coding standards.

/// Lists all bundles for a specific user, with related entities expanded.
pub async fn list_expanded_bundles_for_user(
    pool: &sqlx::postgres::PgPool,
    user_id: sqlx::types::Uuid,
    search_pattern: &str,
    sort_by: &str,
    sort_order: &str,
    limit: i64,
    offset: i64,
) -> std::result::Result<std::vec::Vec<crate::types::expanded_bundle::ExpandedBundle>, sqlx::Error> {
    // Step a: Fetch paginated, filtered, and sorted bundles for the user.
    let sort_by_lc = sort_by.to_lowercase();
    let sort_order_lc = sort_order.to_lowercase();
    let bundles: std::vec::Vec<crate::db::bundles::Bundle> = match (sort_by_lc.as_str(), sort_order_lc.as_str()) {
        ("name", "asc") => sqlx::query_as!(
            crate::db::bundles::Bundle,
            r#"
            SELECT id, user_id, name, description, style_id, document_ids, asset_ids, format_ids, created_at, updated_at
            FROM bundles
            WHERE user_id = $1 AND name ILIKE $2
            ORDER BY name ASC
            LIMIT $3 OFFSET $4
            "#,
            user_id,
            search_pattern,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?,
        ("name", "desc") => sqlx::query_as!(
            crate::db::bundles::Bundle,
            r#"
            SELECT id, user_id, name, description, style_id, document_ids, asset_ids, format_ids, created_at, updated_at
            FROM bundles
            WHERE user_id = $1 AND name ILIKE $2
            ORDER BY name DESC
            LIMIT $3 OFFSET $4
            "#,
            user_id,
            search_pattern,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?,
        ("created_at", "asc") => sqlx::query_as!(
            crate::db::bundles::Bundle,
            r#"
            SELECT id, user_id, name, description, style_id, document_ids, asset_ids, format_ids, created_at, updated_at
            FROM bundles
            WHERE user_id = $1 AND name ILIKE $2
            ORDER BY created_at ASC
            LIMIT $3 OFFSET $4
            "#,
            user_id,
            search_pattern,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?,
        ("created_at", "desc") => sqlx::query_as!(
            crate::db::bundles::Bundle,
            r#"
            SELECT id, user_id, name, description, style_id, document_ids, asset_ids, format_ids, created_at, updated_at
            FROM bundles
            WHERE user_id = $1 AND name ILIKE $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            user_id,
            search_pattern,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?,
        ("updated_at", "asc") => sqlx::query_as!(
            crate::db::bundles::Bundle,
            r#"
            SELECT id, user_id, name, description, style_id, document_ids, asset_ids, format_ids, created_at, updated_at
            FROM bundles
            WHERE user_id = $1 AND name ILIKE $2
            ORDER BY updated_at ASC
            LIMIT $3 OFFSET $4
            "#,
            user_id,
            search_pattern,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?,
        ("updated_at", "desc") | (_, _) => sqlx::query_as!(
            crate::db::bundles::Bundle,
            r#"
            SELECT id, user_id, name, description, style_id, document_ids, asset_ids, format_ids, created_at, updated_at
            FROM bundles
            WHERE user_id = $1 AND name ILIKE $2
            ORDER BY updated_at DESC
            LIMIT $3 OFFSET $4
            "#,
            user_id,
            search_pattern,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?,
    };

    if bundles.is_empty() {
        return std::result::Result::Ok(std::vec::Vec::new());
    }

    // Step b: Collect all unique `style_id`s, `document_id`s, and `asset_id`s.
    let mut style_ids = std::collections::HashSet::new();
    let mut document_ids_set = std::collections::HashSet::new();
    let mut asset_ids_set = std::collections::HashSet::new();
    let mut format_ids_set = std::collections::HashSet::new();

    for bundle in &bundles {
        style_ids.insert(bundle.style_id);
        for doc_id in &bundle.document_ids {
            document_ids_set.insert(*doc_id);
        }
        for asset_id in &bundle.asset_ids {
            asset_ids_set.insert(*asset_id);
        }
        for format_id in &bundle.format_ids {
            format_ids_set.insert(*format_id);
        }
    }

    let style_ids_vec: std::vec::Vec<sqlx::types::Uuid> = style_ids.into_iter().collect();
    let document_ids_vec: std::vec::Vec<sqlx::types::Uuid> = document_ids_set.into_iter().collect();
    let asset_ids_vec: std::vec::Vec<sqlx::types::Uuid> = asset_ids_set.into_iter().collect();
    let format_ids_vec: std::vec::Vec<sqlx::types::Uuid> = format_ids_set.into_iter().collect();

    // Step c: Fetch required Styles, Documents, and Assets in batch queries.
    let styles_map: std::collections::HashMap<sqlx::types::Uuid, crate::db::styles::Style> =
        if !style_ids_vec.is_empty() {
            sqlx::query_as!(
                crate::db::styles::Style,
                r#"SELECT id, user_id, name, html_url, screenshot_url, is_public, created_at, updated_at FROM styles WHERE id = ANY($1)"#,
                &style_ids_vec
            )
                .fetch_all(pool)
                .await?
                .into_iter()
                .map(|s| (s.id, s))
                .collect()
        } else {
            std::collections::HashMap::new()
        };

    let mut documents_map: std::collections::HashMap<sqlx::types::Uuid, crate::db::documents::Document> = std::collections::HashMap::new();
    if !document_ids_vec.is_empty() {
        documents_map = sqlx::query!(
                r#"SELECT 
                    d.id as "id!",
                    d.user_id,
                    d.title as "title!",
                    d.content as "content!",
                    d.sources as "sources!",
                    d.status as "status!",
                    d.created_at as "created_at!",
                    d.updated_at as "updated_at!",
                    d.is_public as "is_public!",
                    d.is_task as "is_task!",
                    d.include_research,
                    d.collection_id
                   FROM documents d 
                   WHERE d.id = ANY($1)"#,
                &document_ids_vec
            )
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|row| {
                let doc = crate::db::documents::Document {
                    id: row.id,
                    user_id: row.user_id,
                    title: row.title,
                    content: row.content,
                    sources: row.sources,
                    status: row.status,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    is_public: row.is_public,
                    is_task: row.is_task,
                    include_research: row.include_research.and_then(|s| s.parse().ok()),
                    collection_id: row.collection_id,
                };
                (doc.id, doc)
            })
            .collect::<std::collections::HashMap<_, _>>();
    }

    let assets_map: std::collections::HashMap<sqlx::types::Uuid, crate::db::assets::Asset> =
        if !asset_ids_vec.is_empty() {
            sqlx::query_as!(
                crate::db::assets::Asset,
                r#"SELECT id, user_id, name, type, gcs_object_name, url, collection_id, metadata, created_at, updated_at, is_public FROM assets WHERE id = ANY($1) AND is_public = FALSE"#,
                &asset_ids_vec
            )
                .fetch_all(pool)
                .await?
                .into_iter()
                .map(|a| (a.id, a))
                .collect()
        } else {
            std::collections::HashMap::new()
        };

    let formats_map: std::collections::HashMap<sqlx::types::Uuid, crate::db::custom_creative_formats::CustomCreativeFormat> =
        if !format_ids_vec.is_empty() {
            sqlx::query_as!(
                crate::db::custom_creative_formats::CustomCreativeFormat,
                r#"SELECT id, user_id, name, description, width, height, creative_type, json_schema, is_public, metadata, created_at, updated_at FROM custom_creative_formats WHERE id = ANY($1)"#,
                &format_ids_vec
            )
                .fetch_all(pool)
                .await?
                .into_iter()
                .map(|f| (f.id, f))
                .collect()
        } else {
            std::collections::HashMap::new()
        };

    // Step d: Iterate through original Bundles and construct ExpandedBundle objects.
    let mut expanded_bundles = std::vec::Vec::with_capacity(bundles.len());
    for bundle in bundles {
        let style = match styles_map.get(&bundle.style_id) {
            Some(s) => s.clone(),
            None => {
                // If a style_id exists in bundle but not in styles table, return an error.
                let err_msg = format!("Style with ID {} not found for bundle ID {}.", bundle.style_id, bundle.id);
                return std::result::Result::Err(sqlx::Error::Decode(Box::new(
                    std::io::Error::new(std::io::ErrorKind::NotFound, err_msg)
                )));
            }
        };

        let documents: std::vec::Vec<crate::db::documents::Document> = bundle
            .document_ids
            .iter()
            .filter_map(|doc_id| documents_map.get(doc_id).cloned())
            .collect();

        let assets: std::vec::Vec<crate::db::assets::Asset> = bundle
            .asset_ids
            .iter()
            .filter_map(|asset_id| assets_map.get(asset_id).cloned())
            .collect();

        let formats: std::vec::Vec<crate::db::custom_creative_formats::CustomCreativeFormat> = bundle
            .format_ids
            .iter()
            .filter_map(|format_id| formats_map.get(format_id).cloned())
            .collect();

        expanded_bundles.push(crate::types::expanded_bundle::ExpandedBundle {
            id: bundle.id,
            user_id: bundle.user_id,
            name: bundle.name,
            description: bundle.description,
            style,
            documents,
            assets,
            formats, // Corrected from format_ids
            created_at: bundle.created_at,
            updated_at: bundle.updated_at,
        });
    }

    std::result::Result::Ok(expanded_bundles)
}
