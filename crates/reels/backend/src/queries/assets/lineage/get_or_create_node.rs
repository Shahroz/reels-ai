use sqlx::types::Uuid;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct StudioNodeRow {
    pub id: Uuid,
    pub journey_id: Uuid,
    pub asset_id: Uuid,
    pub parent_node_id: Option<Uuid>,
}

pub async fn get_or_create_node(
    pool: &sqlx::PgPool,
    journey_id: Uuid,
    asset_id: Uuid,
    parent_node_id: Option<Uuid>,
) -> Result<StudioNodeRow, sqlx::Error> {
    if let Some(row) = sqlx::query_as!(
        StudioNodeRow,
        r#"SELECT id, journey_id, asset_id, parent_node_id FROM studio_nodes WHERE journey_id = $1 AND asset_id = $2 LIMIT 1"#,
        journey_id,
        asset_id
    )
    .fetch_optional(pool)
    .await? {
        return Ok(row);
    }

    let created = sqlx::query_as!(
        StudioNodeRow,
        r#"INSERT INTO studio_nodes (journey_id, asset_id, parent_node_id) VALUES ($1, $2, $3)
           RETURNING id, journey_id, asset_id, parent_node_id"#,
        journey_id,
        asset_id,
        parent_node_id
    )
    .fetch_one(pool)
    .await?;

    Ok(created)
}

