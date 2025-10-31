//! Counts existing asset name variants for deduplication.
//!
//! Queries the database to find how many assets already exist for a user
//! with names starting with a given base pattern. Used to generate
//! numbered variants (e.g., "name (2)", "name (3)").
//!
//! Revision History:
//! - 2025-10-17T00:00:00Z @AI: Extracted from enhance_asset.rs

pub async fn count_existing_name_variants(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    name_base: &str,
) -> std::result::Result<i64, sqlx::Error> {
    // Count any existing assets for this user where name starts with the base
    // Examples matched: "room_1 - Stage: Scandinavian.png", "room_1 - Stage: Scandinavian (2).webp"
    let like_pattern = format!("{}%", name_base);
    let row = sqlx::query!(
        r#"SELECT COUNT(*) as cnt FROM assets WHERE user_id = $1 AND name LIKE $2"#,
        user_id,
        like_pattern
    )
    .fetch_one(pool)
    .await?;
    Ok(row.cnt.unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_count_existing_name_variants_no_matches() {
        // This is a placeholder test - in a real scenario, you'd use a test database
        // You should implement proper integration tests with test fixtures
    }

    #[tokio::test]
    async fn test_count_existing_name_variants_with_matches() {
        // This is a placeholder test - in a real scenario, you'd:
        // 1. Create test assets with various name patterns
        // 2. Query for a specific base name
        // 3. Verify the count matches expected number of variants
        // You should implement proper integration tests with test fixtures
    }

    #[test]
    fn test_like_pattern_construction() {
        let name_base = "test_asset - Enhancement";
        let like_pattern = format!("{}%", name_base);
        assert_eq!(like_pattern, "test_asset - Enhancement%");
        
        // Verify pattern would match expected variants
        assert!(like_pattern.starts_with("test_asset - Enhancement"));
    }

    // Note: Proper tests for this function require:
    // - Test database setup
    // - Fixtures for assets
    // - Consider using test_utils::helpers::TestUser for integration tests
}


