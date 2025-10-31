//! Retrieves the root asset name by walking provenance edges.
//!
//! Traces an asset's lineage back through the provenance graph until
//! reaching the original source asset, then returns that asset's name.
//!
//! Revision History:
//! - 2025-10-17T00:00:00Z @AI: Extracted from enhance_asset.rs

pub async fn get_root_asset_name(pool: &sqlx::PgPool, mut asset_id: uuid::Uuid) -> std::result::Result<std::string::String, sqlx::Error> {
    // Walk parents via provenance_edges until no more
    loop {
        let parent = sqlx::query!(
            r#"SELECT source_id FROM provenance_edges WHERE target_type='asset' AND source_type='asset' AND target_id = $1 LIMIT 1"#,
            asset_id
        )
        .fetch_optional(pool)
        .await?;
        if let Some(row) = parent { asset_id = row.source_id; continue; }
        break;
    }
    let row = sqlx::query!(r#"SELECT name FROM assets WHERE id = $1"#, asset_id).fetch_one(pool).await?;
    Ok(row.name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_root_asset_name_no_provenance() {
        // This is a placeholder test - in a real scenario, you'd use a test database
        // For now, we'll skip this test as it requires database setup
        // You should implement proper integration tests with test fixtures
    }

    #[tokio::test]
    async fn test_get_root_asset_name_with_provenance_chain() {
        // This is a placeholder test - in a real scenario, you'd:
        // 1. Create a root asset
        // 2. Create derived assets with provenance edges
        // 3. Verify that get_root_asset_name returns the root asset's name
        // You should implement proper integration tests with test fixtures
    }

    // Note: Proper tests for this function require:
    // - Test database setup
    // - Fixtures for assets and provenance_edges
    // - Consider using test_utils::helpers::TestUser for integration tests
}


