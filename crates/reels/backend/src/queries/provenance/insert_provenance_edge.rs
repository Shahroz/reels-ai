#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "id")]
pub enum NodeRef {
    #[serde(rename = "asset")] Asset(uuid::Uuid),
    #[serde(rename = "document")] Document(uuid::Uuid),
    #[serde(rename = "video")] Video(uuid::Uuid),
}

impl NodeRef {
    fn split(&self) -> (&'static str, uuid::Uuid) {
        match self {
            NodeRef::Asset(id) => ("asset", *id),
            NodeRef::Document(id) => ("document", *id),
            NodeRef::Video(id) => ("video", *id),
        }
    }
}

pub async fn insert_provenance_edge(
    pool: &sqlx::PgPool,
    source: &NodeRef,
    target: &NodeRef,
    relation_type: &str,
    params: &serde_json::Value,
) -> Result<(), sqlx::Error> {
    let (source_type, source_id) = source.split();
    let (target_type, target_id) = target.split();

    sqlx::query!(
        r#"INSERT INTO provenance_edges (source_type, source_id, target_type, target_id, relation_type, params)
           VALUES ($1, $2, $3, $4, $5, $6)
           ON CONFLICT (source_type, source_id, target_type, target_id) DO NOTHING"#,
        source_type,
        source_id,
        target_type,
        target_id,
        relation_type,
        params
    )
    .execute(pool)
    .await?;

    Ok(())
}


