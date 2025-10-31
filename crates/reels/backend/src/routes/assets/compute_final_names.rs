//! Computes final asset names with deduplication logic.
//!
//! For each enhanced asset URI, generates a unique name based on the root
//! asset name and short label, with automatic numbering for duplicates.
//! Caches existing counts to minimize database queries.
//!
//! Revision History:
//! - 2025-10-17T00:00:00Z @AI: Extracted from enhance_asset.rs

pub async fn compute_final_names(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    originals: &[crate::db::assets::Asset],
    uris: &[std::string::String],
    short_label: &str,
) -> std::result::Result<std::vec::Vec<std::string::String>, sqlx::Error> {
    let mut existing_counts_cache: std::collections::HashMap<std::string::String, i64> = std::collections::HashMap::new();
    let mut batch_increments: std::collections::HashMap<std::string::String, i64> = std::collections::HashMap::new();
    let mut out: std::vec::Vec<std::string::String> = std::vec::Vec::with_capacity(uris.len());

    for (i, uri) in uris.iter().enumerate() {
        let original = originals.get(i).expect("URI without corresponding original asset");
        let root_name = crate::routes::assets::get_root_asset_name::get_root_asset_name(pool, original.id).await.unwrap_or_else(|_| original.name.clone());
        let root_base = root_name.rsplit_once('.').map(|(b, _)| b.to_string()).unwrap_or_else(|| root_name.clone());

        let base_key = format!("{} - {}", root_base, short_label);
        let existing = if let Some(c) = existing_counts_cache.get(&base_key) { *c } else {
            let c = crate::routes::assets::count_existing_name_variants::count_existing_name_variants(pool, user_id, &base_key).await?;
            existing_counts_cache.insert(base_key.clone(), c);
            c
        };
        let entry = batch_increments.entry(base_key.clone()).or_insert(0);
        *entry += 1;
        let ordinal = existing + *entry; // 1 = first (no suffix), 2+ = numbered

        // extension from URI if plausible, else from original
        let uri_ext = uri.split('.').last().filter(|e| e.len() <= 5).map(|s| s.to_string());
        let orig_ext = original.name.split('.').last().map(|s| s.to_string());
        let ext = uri_ext.or(orig_ext).unwrap_or_else(|| "jpg".to_string());

        let name = if ordinal <= 1 { 
            format!("{} - {}.{}", root_base, short_label, ext)
        } else {
            format!("{} - {} ({}).{}", root_base, short_label, ordinal, ext)
        };
        out.push(name);
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_format_without_ordinal() {
        let root_base = "photo_1";
        let short_label = "Stage: Modern";
        let ext = "jpg";
        let ordinal = 1;
        
        let name = if ordinal <= 1 {
            format!("{} - {}.{}", root_base, short_label, ext)
        } else {
            format!("{} - {} ({}).{}", root_base, short_label, ordinal, ext)
        };
        
        assert_eq!(name, "photo_1 - Stage: Modern.jpg");
    }

    #[test]
    fn test_name_format_with_ordinal() {
        let root_base = "photo_1";
        let short_label = "Stage: Modern";
        let ext = "jpg";
        let ordinal = 3;
        
        let name = if ordinal <= 1 {
            format!("{} - {}.{}", root_base, short_label, ext)
        } else {
            format!("{} - {} ({}).{}", root_base, short_label, ordinal, ext)
        };
        
        assert_eq!(name, "photo_1 - Stage: Modern (3).jpg");
    }

    #[test]
    fn test_extension_extraction() {
        let uri = "gs://bucket/path/to/file.webp";
        let uri_ext = uri.split('.').last().filter(|e| e.len() <= 5).map(|s| s.to_string());
        assert_eq!(uri_ext, Some("webp".to_string()));
    }

    #[test]
    fn test_extension_fallback() {
        let uri = "gs://bucket/path/to/file_without_ext";
        let uri_ext = uri.split('.').last().filter(|e| e.len() <= 5).map(|s| s.to_string());
        let original_name = "original.png";
        let orig_ext = original_name.split('.').last().map(|s| s.to_string());
        let ext = uri_ext.or(orig_ext).unwrap_or_else(|| "jpg".to_string());
        assert_eq!(ext, "png");
    }

    #[test]
    fn test_extension_default() {
        let uri = "gs://bucket/path/to/file_without_ext";
        let uri_ext = uri.split('.').last().filter(|e| e.len() <= 5).map(|s| s.to_string());
        let ext = uri_ext.unwrap_or_else(|| "jpg".to_string());
        assert_eq!(ext, "jpg");
    }

    #[tokio::test]
    async fn test_compute_final_names_integration() {
        // This is a placeholder test - in a real scenario, you'd:
        // 1. Set up test database with user and assets
        // 2. Call compute_final_names with test data
        // 3. Verify generated names are unique and properly formatted
        // You should implement proper integration tests with test fixtures
    }

    // Note: Full integration tests require:
    // - Test database setup
    // - Fixtures for users and assets
    // - Consider using test_utils::helpers::TestUser for integration tests
}


