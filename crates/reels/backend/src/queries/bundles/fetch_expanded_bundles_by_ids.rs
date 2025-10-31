//! Defines the `fetch_expanded_bundles_by_ids` database query function.
//!
//! This function retrieves specific bundles by their IDs for a given user,
//! expanding associated style, documents, assets, and formats into their full structures.
//! It returns a `Vec<crate::types::expanded_bundle::ExpandedBundle>`.
//! Adheres to the project's Rust coding standards.

// Revision History
// - 2025-06-02T15:34:19Z @AI: Initial implementation.

pub async fn fetch_expanded_bundles_by_ids(
    pool: &sqlx::postgres::PgPool,
    user_id: sqlx::types::Uuid,
    bundle_ids_to_fetch: &[sqlx::types::Uuid],
) -> std::result::Result<std::vec::Vec<crate::types::expanded_bundle::ExpandedBundle>, sqlx::Error> {
    // Handle empty bundle_ids_to_fetch early.
    if bundle_ids_to_fetch.is_empty() {
        return std::result::Result::Ok(std::vec::Vec::new());
    }

    // Step a: Fetch `crate::db::bundles::Bundle` for the `user_id` and matching `bundle_ids_to_fetch`.
    let bundles: std::vec::Vec<crate::db::bundles::Bundle> = sqlx::query_as!(
        crate::db::bundles::Bundle,
        r#"
        SELECT id, user_id, name, description, style_id, document_ids, asset_ids, format_ids, created_at, updated_at
        FROM bundles
        WHERE user_id = $1 AND id = ANY($2)
        ORDER BY name ASC
        "#,
        user_id,
        bundle_ids_to_fetch
    )
        .fetch_all(pool)
        .await?;

    if bundles.is_empty() {
        return std::result::Result::Ok(std::vec::Vec::new());
    }

    // Step b: Collect all unique `style_id`s, `document_id`s, `asset_id`s, and `format_id`s.
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

    // Step c: Fetch required Styles, Documents, Assets, and Formats in batch queries.
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

    let documents_map: std::collections::HashMap<sqlx::types::Uuid, crate::db::documents::Document> =
        if !document_ids_vec.is_empty() {
            let documents_result = sqlx::query!(
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
                WHERE d.id = ANY($1) AND d.user_id = $2"#,
                &document_ids_vec,
                user_id
            )
                .fetch_all(pool)
                .await?;

            let mut documents_map = std::collections::HashMap::new();
            for row in documents_result {
                documents_map.insert(row.id, crate::db::documents::Document {
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
                });
            }
            documents_map
        } else {
            std::collections::HashMap::new()
        };

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
                let err_msg = format!(
                    "Data integrity error: Style with ID {} not found for bundle ID {}.",
                    bundle.style_id, bundle.id
                );
                return std::result::Result::Err(sqlx::Error::Decode(Box::new(
                    std::io::Error::new(std::io::ErrorKind::NotFound, err_msg),
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
            formats,
            created_at: bundle.created_at,
            updated_at: bundle.updated_at,
        });
    }

    std::result::Result::Ok(expanded_bundles)
    // Function length justification:
    // This function exceeds the 50 LoC guideline primarily due to the multi-step nature of
    // fetching related data (bundles, then styles, documents, assets, formats) and
    // aggregating it. Each step involves a database query and data transformation/collection,
    // contributing to the length. Breaking it down further would obscure the overall workflow.
}

#[cfg(test)]
mod tests {
    // Using fully qualified paths as per guidelines.
    // Note: Testing this function thoroughly requires a test database.
    // The following are placeholder/conceptual tests.

    // #[sqlx::test]
    // async fn test_fetch_expanded_bundles_by_ids_empty_input_ids(
    //     pool: sqlx::postgres::PgPool,
    // ) -> std::result::Result<(), sqlx::Error> {
    //     let user_id = sqlx::types::Uuid::new_v4();
    //     let bundle_ids_to_fetch: [sqlx::types::Uuid; 0] = [];
    //     let result = super::fetch_expanded_bundles_by_ids(&pool, user_id, &bundle_ids_to_fetch).await?;
    //     assert!(result.is_empty());
    //     std::result::Result::Ok(())
    // }

    // #[sqlx::test]
    // async fn test_fetch_expanded_bundles_by_ids_no_matching_bundles(
    //     pool: sqlx::postgres::PgPool,
    // ) -> std::result::Result<(), sqlx::Error> {
    //     let user_id = sqlx::types::Uuid::new_v4();
    //     let non_existent_bundle_id = sqlx::types::Uuid::new_v4();
    //     let bundle_ids_to_fetch = [non_existent_bundle_id];
    //     let result = super::fetch_expanded_bundles_by_ids(&pool, user_id, &bundle_ids_to_fetch).await?;
    //     assert!(result.is_empty());
    //     std::result::Result::Ok(())
    // }

    // Further tests would involve:
    // - Setting up a bundle, style, documents, assets, formats.
    // - Calling fetch_expanded_bundles_by_ids with the bundle's ID and correct user ID.
    // - Verifying the expanded bundle matches expected data.
    // - Testing with a mix of valid and invalid bundle IDs for the user.
    // - Testing the case where a style referenced by a valid bundle is missing (should error).
}