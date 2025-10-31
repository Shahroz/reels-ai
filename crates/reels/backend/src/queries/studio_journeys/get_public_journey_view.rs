//! Fetches the necessary data for a public, read-only view of a Studio Journey.
//!
//! This query retrieves the journey's name and a list of its nodes, including
//! the node ID, asset name, asset URL, and parent node ID. It's designed to
//! provide all necessary information for the public-facing journey viewer.

/// Represents a single node in the public journey view.
#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct PublicJourneyNode {
    pub id: uuid::Uuid,
    pub name: String,
    pub url: String,
    pub parent_node_id: Option<uuid::Uuid>,
}

/// Represents the complete data structure for a public journey view.
#[derive(Debug, serde::Serialize)]
pub struct PublicJourneyView {
    pub name: String,
    pub nodes: Vec<PublicJourneyNode>,
}

/// Fetches the journey name and its nodes by journey ID.
pub async fn get_public_journey_view(
    pool: &sqlx::PgPool,
    journey_id: uuid::Uuid,
) -> Result<Option<PublicJourneyView>, sqlx::Error> {
    // First, get the journey's name.
    let journey_meta = sqlx::query!(
        "SELECT name FROM studio_journeys WHERE id = $1",
        journey_id
    )
    .fetch_optional(pool)
    .await?;

    let name = match journey_meta {
        Some(record) => record.name.unwrap_or_else(|| "Untitled Journey".to_string()),
        None => return Ok(None), // Journey not found
    };

    // Second, get all nodes associated with the journey.
    let nodes = sqlx::query_as!(
        PublicJourneyNode,
        r#"
        SELECT
            n.id,
            a.name,
            a.url,
            n.parent_node_id
        FROM
            studio_nodes AS n
        JOIN
            assets AS a ON n.asset_id = a.id
        WHERE
            n.journey_id = $1
        ORDER BY
            a.created_at ASC
        "#,
        journey_id
    )
    .fetch_all(pool)
    .await?;

    Ok(Some(PublicJourneyView { name, nodes }))
}