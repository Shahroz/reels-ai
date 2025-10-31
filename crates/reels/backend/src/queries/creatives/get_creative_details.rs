//! Query for fetching a single creative by ID with all its related data.
//!
//! Fetches the creative and checks for user access via ownership or sharing.
//! Also fetches related entities like style, assets, documents, etc.

use crate::db;
use crate::queries::organizations::find_active_memberships_for_user;
use crate::routes::creatives::responses::GetCreativeDetails;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug)]
struct CreativeWithAccessDetails {
    id: Uuid,
    name: String,
    collection_id: Option<Uuid>,
    creative_format_id: Uuid,
    style_id: Option<Uuid>,
    document_ids: Option<Vec<Uuid>>,
    asset_ids: Option<Vec<Uuid>>,
    html_url: String,
    draft_url: Option<String>,
    bundle_id: Option<Uuid>,
    screenshot_url: String,
    is_published: bool,
    publish_url: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    creator_email: Option<String>,
    current_user_access_level: Option<String>,
    is_favorite: Option<bool>,
}

/// Fetches a creative by its ID, ensuring the user has access.
/// Returns a `GetCreativeDetails` struct containing the creative and all its relations.
pub async fn get_creative_details(
    pool: &PgPool,
    user_id: Uuid,
    creative_id: Uuid,
) -> anyhow::Result<Option<GetCreativeDetails>> {
    let org_memberships = find_active_memberships_for_user(pool, user_id).await?;
    let org_ids: Vec<Uuid> = org_memberships.into_iter().map(|m| m.organization_id).collect();

    let query_result = sqlx::query_as!(
        CreativeWithAccessDetails,
        r#"
        WITH UserOrgMemberships_CTE AS (
            SELECT organization_id FROM organization_members WHERE user_id = $1 AND status = 'active'
        ),
        CreativeShares_CTE AS (
            SELECT
                os.access_level,
                ROW_NUMBER() OVER (ORDER BY
                    CASE os.access_level
                        WHEN 'editor' THEN 1
                        WHEN 'viewer' THEN 2
                        ELSE 3
                    END
                ) as rn
            FROM object_shares os
            WHERE os.object_type = 'creative' AND os.object_id = $2
              AND (
                    (os.entity_type = 'user' AND entity_id = $1)
                    OR
                    (os.entity_type = 'organization' AND entity_id = ANY($3::UUID[]))
                )
        ),
        CollectionShares_CTE AS (
            SELECT
                os.access_level,
                ROW_NUMBER() OVER (ORDER BY
                    CASE os.access_level
                        WHEN 'editor' THEN 1
                        WHEN 'viewer' THEN 2
                        ELSE 3
                    END
                ) as rn
            FROM object_shares os
            INNER JOIN creatives c ON c.collection_id = os.object_id
            WHERE c.id = $2 AND os.object_type = 'collection'
              AND (
                    (os.entity_type = 'user' AND entity_id = $1)
                    OR
                    (os.entity_type = 'organization' AND entity_id = ANY($3::UUID[]))
                )
        )
        SELECT 
            c.id, c.name, c.collection_id, c.creative_format_id, c.style_id, c.document_ids,
            c.asset_ids, c.html_url, c.draft_url, c.bundle_id,
            c.screenshot_url, c.is_published, c.publish_url,
            c.created_at, c.updated_at,
            u_creator.email AS "creator_email?",
            CASE
                WHEN col.user_id = $1 THEN 'owner'::text
                ELSE COALESCE(
                    (SELECT access_level::text FROM CreativeShares_CTE WHERE rn = 1),
                    (SELECT access_level::text FROM CollectionShares_CTE WHERE rn = 1)
                )
            END AS "current_user_access_level?",
            CASE
                WHEN c.id IN (SELECT entity_id FROM user_favorites WHERE user_id = $1 AND entity_type = 'creative'::favorite_entity_type) THEN true
                ELSE false
            END AS "is_favorite?"
        FROM creatives c
        INNER JOIN collections col ON c.collection_id = col.id
        LEFT JOIN users u_creator ON col.user_id = u_creator.id
        WHERE c.id = $2 AND (
            col.user_id = $1 
            OR EXISTS (SELECT 1 FROM CreativeShares_CTE WHERE rn = 1)
            OR EXISTS (SELECT 1 FROM CollectionShares_CTE WHERE rn = 1)
        )
        "#,
        user_id,      // $1
        creative_id,  // $2
        &org_ids      // $3
    )
    .fetch_optional(pool)
    .await?;

    if let Some(details) = query_result {
        if details.current_user_access_level.is_none() {
            return Ok(None);
        }

        // Fetch related data
        let mut style = None;
        let mut assets = Vec::new();
        let mut documents = Vec::new();
        let mut creative_format = None;
        let mut collection = None;
        let mut bundle = None;

        // Fetch style if present
        if let Some(style_id) = details.style_id {
            if let Ok(Some(style_data)) = sqlx::query_as!(
                db::styles::Style,
                r#"
                SELECT s.*
                FROM styles s
                LEFT JOIN (
                    SELECT os.object_id, os.access_level
                    FROM object_shares os
                    WHERE os.object_type = 'style' AND os.object_id = $1
                    AND (
                        (os.entity_type = 'user' AND os.entity_id = $2)
                        OR
                        (os.entity_type = 'organization' AND os.entity_id = ANY($3::UUID[]))
                    )
                    ORDER BY CASE os.access_level WHEN 'editor' THEN 1 WHEN 'viewer' THEN 2 ELSE 3 END
                    LIMIT 1
                ) es ON s.id = es.object_id
                WHERE s.id = $1 AND (s.user_id = $2 OR s.is_public = true OR es.access_level IS NOT NULL)
                "#,
                style_id,
                user_id,
                &org_ids
            )
            .fetch_optional(pool)
            .await
            {
                style = Some(style_data);
            }
        }

        // Fetch assets if present
        if let Some(asset_ids) = &details.asset_ids {
            if !asset_ids.is_empty() {
                if let Ok(fetched_assets) = sqlx::query_as!(
                    db::assets::Asset,
                    "SELECT id, user_id, name, type, gcs_object_name, url, collection_id, metadata, created_at, updated_at, is_public FROM assets WHERE id = ANY($1)",
                    asset_ids
                )
                .fetch_all(pool)
                .await
                {
                    assets = fetched_assets;
                }
            }
        }

        // Fetch documents if present
        if let Some(document_ids) = &details.document_ids {
            if !document_ids.is_empty() {
                if let Ok(fetched_documents) = sqlx::query_as!(
                    db::documents::Document,
                    "SELECT id, user_id, title, content, sources, status, created_at, updated_at, is_public, is_task, include_research as \"include_research: _\", collection_id FROM documents WHERE id = ANY($1)",
                    document_ids
                )
                .fetch_all(pool)
                .await
                {
                    documents = fetched_documents;
                }
            }
        }

        // Fetch creative format (required)
        if let Ok(Some(format_data)) = sqlx::query_as!(
            db::custom_creative_formats::CustomCreativeFormat,
            r#"SELECT 
                 id, user_id, name, description, width, height, 
                 creative_type AS "creative_type: _", json_schema, metadata, 
                 created_at, updated_at, is_public 
               FROM custom_creative_formats 
               WHERE id = $1"#,
            details.creative_format_id
        )
        .fetch_optional(pool)
        .await
        {
            creative_format = Some(format_data);
        }

        // Fetch collection if present
        if let Some(collection_id) = details.collection_id {
            if let Ok(Some(collection_data)) = sqlx::query_as!(
                db::collections::Collection,
                r#"
                SELECT col.*
                FROM collections col
                LEFT JOIN (
                    SELECT os.object_id, os.access_level
                    FROM object_shares os
                    WHERE os.object_type = 'collection' AND os.object_id = $1
                    AND (
                        (os.entity_type = 'user' AND os.entity_id = $2)
                        OR
                        (os.entity_type = 'organization' AND os.entity_id = ANY($3::UUID[]))
                    )
                    ORDER BY CASE os.access_level WHEN 'editor' THEN 1 WHEN 'viewer' THEN 2 ELSE 3 END
                    LIMIT 1
                ) es ON col.id = es.object_id
                WHERE col.id = $1 AND (col.user_id = $2 OR es.access_level IS NOT NULL)
                "#,
                collection_id,
                user_id,
                &org_ids
            )
            .fetch_optional(pool)
            .await
            {
                collection = Some(collection_data);
            }
        }

        // Fetch bundle if present
        if let Some(bundle_id) = details.bundle_id {
            if let Ok(Some(bundle_data)) = crate::queries::bundles::find_bundle_by_id::find_bundle_by_id(pool, bundle_id).await {
                bundle = Some(bundle_data);
            }
        }

        let response = GetCreativeDetails {
            creative: db::creatives::Creative {
                id: details.id,
                name: details.name,
                collection_id: details.collection_id,
                creative_format_id: details.creative_format_id,
                style_id: details.style_id,
                document_ids: details.document_ids,
                asset_ids: details.asset_ids,
                html_url: details.html_url,
                draft_url: details.draft_url,
                bundle_id: details.bundle_id,
                screenshot_url: details.screenshot_url,
                is_published: details.is_published,
                publish_url: details.publish_url,
                created_at: details.created_at,
                updated_at: details.updated_at,
            },
            creator_email: details.creator_email,
            current_user_access_level: details.current_user_access_level,
            is_favorite: details.is_favorite.unwrap_or(false),
            style,
            assets,
            documents,
            creative_format,
            collection,
            bundle,
        };
        Ok(Some(response))
    } else {
        Ok(None)
    }
}