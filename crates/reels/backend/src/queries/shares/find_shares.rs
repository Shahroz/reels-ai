//! Finds and lists object shares based on a set of filter criteria.
//!
//! This function constructs a dynamic query to filter shares by object, entity,
//! or other parameters. It joins with `users` and `organizations` tables
//! to enrich the response with entity names.
//! Returns a vector of `ObjectShare` structs.

use crate::db::shares::ObjectShare;
use serde::Deserialize;
use sqlx::PgPool;
use sqlx_conditional_queries::conditional_query_as;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct ShareFilter<'a> {
    pub object_id: Option<Uuid>,
    pub object_type: Option<&'a str>,
    pub entity_id: Option<Uuid>,
    pub entity_type: Option<&'a str>,
}

pub async fn find_shares(
    pool: &PgPool,
    filter: &ShareFilter<'_>,
) -> Result<Vec<ObjectShare>, sqlx::Error> {
    let object_id = filter.object_id;
    let object_type = filter.object_type;
    let entity_id = filter.entity_id;
    let entity_type = filter.entity_type;

    conditional_query_as!(
        ObjectShare,
        r#"
        SELECT
            os.id, os.object_id, os.object_type, os.entity_id, os.entity_type, os.access_level,
            os.created_at, os.updated_at,
            CASE
                WHEN os.entity_type = 'user'::object_share_entity_type THEN u.email
                WHEN os.entity_type = 'organization'::object_share_entity_type THEN o.name
                ELSE NULL
            END AS entity_name
        FROM object_shares os
        LEFT JOIN users u ON os.entity_id = u.id AND os.entity_type = 'user'::object_share_entity_type
        LEFT JOIN organizations o ON os.entity_id = o.id AND os.entity_type = 'organization'::object_share_entity_type
        WHERE 1=1
        "#,
        "AND os.object_id = {object_id} AND os.object_type = {object_type}"
            if object_id.is_some() && object_type.is_some(),
        "AND os.entity_id = {entity_id} AND os.entity_type = {entity_type}"
            if entity_id.is_some() && entity_type.is_some(),
        #object_id = match &object_id { _ => "{object_id}" },
        #object_type = match &object_type { _ => "{object_type}" },
        #entity_id = match &entity_id { _ => "{entity_id}" },
        #entity_type = match &entity_type { _ => "{entity_type}" },
    )
        .fetch_all(pool)
        .await
}
