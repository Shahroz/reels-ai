use sqlx::types::Uuid;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct StudioJourneyRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub root_asset_id: Option<Uuid>,
}

pub async fn get_or_create_journey(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    root_asset_id: Uuid,
) -> Result<StudioJourneyRow, sqlx::Error> {
    if let Some(row) = sqlx::query_as!(
        StudioJourneyRow,
        r#"SELECT id, user_id, root_asset_id FROM studio_journeys WHERE user_id = $1 AND root_asset_id = $2 LIMIT 1"#,
        user_id,
        root_asset_id
    )
    .fetch_optional(pool)
    .await? {
        return Ok(row);
    }

    let created = sqlx::query_as!(
        StudioJourneyRow,
        r#"INSERT INTO studio_journeys (user_id, root_asset_id) VALUES ($1, $2) RETURNING id, user_id, root_asset_id"#,
        user_id,
        root_asset_id
    )
    .fetch_one(pool)
    .await?;

    Ok(created)
}

