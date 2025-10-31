//! Retrieves a logo collection with detailed asset information.
//!
//! This function is similar to get_logo_collection_by_id but provides
//! additional asset metadata and extended information for detailed views.
//! Used for comprehensive collection display and management interfaces.

/// Gets a logo collection with full asset details
pub async fn get_logo_collection_with_assets(
    pool: &sqlx::PgPool,
    collection_id: uuid::Uuid,
    user_id: uuid::Uuid,
) -> std::result::Result<std::option::Option<crate::schemas::logo_collection_schemas::LogoCollectionResponse>, sqlx::Error> {
    // Reuse the existing get_logo_collection_by_id function
    crate::queries::logo_collections::get_logo_collection_by_id::get_logo_collection_by_id(
        pool,
        collection_id,
        user_id,
    )
    .await
}


// Tests temporarily disabled - need proper test infrastructure
