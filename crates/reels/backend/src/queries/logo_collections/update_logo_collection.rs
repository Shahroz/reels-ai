//! Updates an existing logo collection.
//!
//! This function updates the name and description of a logo collection.
//! Only the collection owner can update their collections.
//! Returns the updated collection record with new timestamps.

/// Updates a logo collection's name and description
pub async fn update_logo_collection(
    pool: &sqlx::PgPool,
    collection_id: uuid::Uuid,
    user_id: uuid::Uuid,
    name: std::option::Option<&str>,
    description: std::option::Option<std::option::Option<&str>>,
) -> std::result::Result<std::option::Option<crate::db::logo_collection::LogoCollection>, sqlx::Error> {
    // Build dynamic update query based on provided fields
    let mut set_clauses = std::vec::Vec::new();
    let mut param_count = 3; // Starting from $3 (after id and user_id)

    if name.is_some() {
        set_clauses.push(std::format!("name = ${}", param_count));
        param_count += 1;
    }

    if description.is_some() {
        set_clauses.push(std::format!("description = ${}", param_count));
        param_count += 1;
    }

    if set_clauses.is_empty() {
        // No updates to make, return existing collection
        let collection = sqlx::query!(
            "SELECT id, user_id, name, description, created_at, updated_at FROM logo_collections WHERE id = $1 AND user_id = $2",
            collection_id,
            user_id
        )
        .fetch_optional(pool)
        .await?;

        if let std::option::Option::Some(row) = collection {
            std::result::Result::Ok(std::option::Option::Some(crate::db::logo_collection::LogoCollection {
                id: row.id,
                user_id: row.user_id,
                name: row.name,
                description: row.description,
                created_at: row.created_at,
                updated_at: row.updated_at,
            }))
        } else {
            std::result::Result::Ok(std::option::Option::None)
        }
    } else {
        // Construct and execute update query
        let set_clause = set_clauses.join(", ");
        let query = std::format!(
            "UPDATE logo_collections SET {}, updated_at = NOW() WHERE id = $1 AND user_id = $2 RETURNING id, user_id, name, description, created_at, updated_at",
            set_clause
        );

        if let (std::option::Option::Some(name_val), std::option::Option::Some(desc_val)) = (name, description) {
            let collection = sqlx::query!(
                "UPDATE logo_collections SET name = $3, description = $4, updated_at = NOW() WHERE id = $1 AND user_id = $2 RETURNING id, user_id, name, description, created_at, updated_at",
                collection_id,
                user_id,
                name_val,
desc_val
            )
            .fetch_optional(pool)
            .await?;

            if let std::option::Option::Some(row) = collection {
                std::result::Result::Ok(std::option::Option::Some(crate::db::logo_collection::LogoCollection {
                    id: row.id,
                    user_id: row.user_id,
                    name: row.name,
                    description: row.description,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                }))
            } else {
                std::result::Result::Ok(std::option::Option::None)
            }
        } else if let std::option::Option::Some(name_val) = name {
            let collection = sqlx::query!(
                "UPDATE logo_collections SET name = $3, updated_at = NOW() WHERE id = $1 AND user_id = $2 RETURNING id, user_id, name, description, created_at, updated_at",
                collection_id,
                user_id,
                name_val
            )
            .fetch_optional(pool)
            .await?;

            if let std::option::Option::Some(row) = collection {
                std::result::Result::Ok(std::option::Option::Some(crate::db::logo_collection::LogoCollection {
                    id: row.id,
                    user_id: row.user_id,
                    name: row.name,
                    description: row.description,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                }))
            } else {
                std::result::Result::Ok(std::option::Option::None)
            }
        } else if let std::option::Option::Some(desc_val) = description {
            let collection = sqlx::query!(
                "UPDATE logo_collections SET description = $3, updated_at = NOW() WHERE id = $1 AND user_id = $2 RETURNING id, user_id, name, description, created_at, updated_at",
                collection_id,
                user_id,
desc_val
            )
            .fetch_optional(pool)
            .await?;

            if let std::option::Option::Some(row) = collection {
                std::result::Result::Ok(std::option::Option::Some(crate::db::logo_collection::LogoCollection {
                    id: row.id,
                    user_id: row.user_id,
                    name: row.name,
                    description: row.description,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                }))
            } else {
                std::result::Result::Ok(std::option::Option::None)
            }
        } else {
            std::result::Result::Ok(std::option::Option::None)
        }
    }
}


// Tests temporarily disabled - need proper test infrastructure
