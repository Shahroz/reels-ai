use sqlx::types::Uuid;
use crate::queries::documents::lineage::types::{DocumentGraph, DocumentGraphEdge, DocumentGraphNode, DocumentDerivationParams};

fn derive_short_label_from_transformation_prompt(prompt: &str) -> Option<String> {
    let p = prompt.to_lowercase();
    let contains = |needle: &str| p.contains(needle);

    // Common content transformation patterns
    if contains("formal") || contains("professional") { return Some("Formalize".to_string()); }
    if contains("casual") || contains("conversational") { return Some("Casualize".to_string()); }
    if contains("summarize") || contains("summary") { return Some("Summary".to_string()); }
    if contains("expand") || contains("elaborate") { return Some("Expand".to_string()); }
    if contains("translate") { return Some("Translate".to_string()); }
    if contains("rewrite") || contains("rephrase") { return Some("Rewrite".to_string()); }
    if contains("simplify") || contains("simple") { return Some("Simplify".to_string()); }
    if contains("technical") || contains("detailed") { return Some("Technical".to_string()); }
    if contains("creative") || contains("engaging") { return Some("Creative".to_string()); }
    if contains("concise") || contains("brief") { return Some("Concise".to_string()); }
    if contains("bullet") || contains("list") { return Some("List Format".to_string()); }
    if contains("paragraph") || contains("prose") { return Some("Prose".to_string()); }
    if contains("email") { return Some("Email Format".to_string()); }
    if contains("blog") || contains("article") { return Some("Blog Post".to_string()); }
    if contains("social media") || contains("tweet") { return Some("Social".to_string()); }
    if contains("marketing") || contains("promotional") { return Some("Marketing".to_string()); }
    if contains("academic") || contains("scholarly") { return Some("Academic".to_string()); }
    if contains("fix") || contains("correct") || contains("grammar") { return Some("Correction".to_string()); }
    
    None
}

fn fallback_label_from_prompt(prompt: &str) -> String {
    let s = prompt.trim();
    if s.is_empty() { return "Transformed".to_string(); }
    let max_len = 28usize;
    let mut out = s.chars().take(max_len).collect::<String>();
    if s.len() > out.len() { out.push_str("…"); }
    // Capitalize first letter
    if let Some(first) = out.get(0..1) {
        out.replace_range(0..1, &first.to_uppercase().to_string());
    }
    out
}

fn create_content_preview(content: &str, max_chars: usize) -> String {
    if content.len() <= max_chars {
        content.to_string()
    } else {
        let truncated = content.chars().take(max_chars).collect::<String>();
        format!("{}...", truncated)
    }
}

pub async fn get_graph_for_document(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    document_id: Uuid,
) -> Result<DocumentGraph, sqlx::Error> {
    // Ensure the document belongs to the user or is public
    let owner_row = sqlx::query!(r#"SELECT user_id, is_public FROM documents WHERE id = $1"#, document_id)
        .fetch_one(pool)
        .await?;
    if owner_row.user_id != Some(user_id) && !owner_row.is_public { 
        return Err(sqlx::Error::RowNotFound); 
    }

    // Use recursive CTE to get full closure (ancestors+descendants) from the seed
    // Only include derived documents that belong to the requesting user
    let all_ids_rows = sqlx::query!(
        r#"
        WITH RECURSIVE ancestors(id) AS (
            SELECT source_id FROM provenance_edges WHERE target_type = 'document' AND source_type = 'document' AND target_id = $1
            UNION
            SELECT d.source_id FROM provenance_edges d JOIN ancestors a ON d.target_id = a.id
             WHERE d.target_type = 'document' AND d.source_type = 'document'
        ),
        descendants(id) AS (
            SELECT target_id FROM provenance_edges WHERE source_type = 'document' AND target_type = 'document' AND source_id = $1
            UNION
            SELECT d.target_id FROM provenance_edges d JOIN descendants c ON d.source_id = c.id
             WHERE d.source_type = 'document' AND d.target_type = 'document'
        ),
        all_ids(id) AS (
            SELECT $1::uuid
            UNION SELECT id FROM ancestors
            UNION SELECT id FROM descendants
        )
        SELECT id FROM all_ids
        "#,
        document_id
    )
    .fetch_all(pool)
    .await?;
    let mut document_ids: Vec<Uuid> = all_ids_rows.into_iter().filter_map(|r| r.id).collect();
    if !document_ids.contains(&document_id) { 
        document_ids.push(document_id); 
    }

    // Identify a provisional root from the first closure
    let edges_seed = sqlx::query!(
        r#"SELECT source_id, target_id FROM provenance_edges
            WHERE (source_type='document' AND target_type='document')
              AND (source_id = ANY($1) OR target_id = ANY($1))"#,
        &document_ids[..]
    )
    .fetch_all(pool)
    .await?;

    let mut root = document_id;
    for r in &document_ids {
        let has_incoming = edges_seed.iter().any(|e| e.target_id == *r);
        if !has_incoming { 
            root = *r; 
            break; 
        }
    }

    // Recompute closure strictly as all descendants from the root (for complete siblings view)
    // Only include derived documents that belong to the requesting user
    let all_desc_rows = sqlx::query!(
        r#"
        WITH RECURSIVE descendants(id) AS (
            SELECT $1::uuid
            UNION
            SELECT d.target_id FROM provenance_edges d 
            JOIN descendants c ON d.source_id = c.id
            JOIN documents doc ON doc.id = d.target_id
             WHERE d.source_type='document' AND d.target_type='document'
               AND (doc.user_id = $2 OR doc.is_public = true)
        )
        SELECT id FROM descendants
        "#,
        root,
        user_id
    )
    .fetch_all(pool)
    .await?;
    let document_ids: Vec<Uuid> = all_desc_rows.into_iter().filter_map(|r| r.id).collect();

    // Fetch edges with params and child created_at within the closure
    // Only include edges where the target document belongs to the requesting user or is public
    let edges_ext = sqlx::query!(
        r#"SELECT d.source_id, d.target_id, d.params, child.created_at as child_created_at
            FROM provenance_edges d
            JOIN documents child ON child.id = d.target_id
            WHERE d.source_type='document' AND d.target_type='document'
              AND (d.source_id = ANY($1) OR d.target_id = ANY($1))
              AND (child.user_id = $2 OR child.is_public = true)"#,
        &document_ids[..],
        user_id
    )
    .fetch_all(pool)
    .await?;

    let all_edges: Vec<DocumentGraphEdge> = edges_ext
        .iter()
        .map(|r| DocumentGraphEdge { 
            source_document_id: r.source_id, 
            derived_document_id: r.target_id 
        })
        .collect();

    // Fetch node details - include root document and user-owned derived documents
    let nodes_rows = sqlx::query!(
        r#"SELECT id, title, content, created_at FROM documents 
           WHERE id = ANY($1) 
           AND (id = $2 OR user_id = $3 OR is_public = true)"#,
        &document_ids[..],
        root,
        user_id
    )
    .fetch_all(pool)
    .await?;

    // Build parent links and transformation prompts for label derivation
    use std::collections::HashMap;
    let mut parent_by_child: HashMap<Uuid, Uuid> = HashMap::new();
    let mut prompt_by_child: HashMap<Uuid, String> = HashMap::new();
    for r in &edges_ext {
        parent_by_child.insert(r.target_id, r.source_id);
        let params = &r.params;
        // Try to parse as DocumentDerivationParams
        if let Ok(enum_params) = serde_json::from_value::<DocumentDerivationParams>(params.clone()) {
            if let DocumentDerivationParams::ContentTransformation(transform) = enum_params {
                prompt_by_child.insert(r.target_id, transform.transformation_prompt);
                continue;
            }
        }
        // Fallback: try to extract transformation_prompt directly from params
        if let Some(prompt) = params.get("transformation_prompt").and_then(|p| p.as_str()) {
            prompt_by_child.insert(r.target_id, prompt.to_string());
        }
    }

    // Get root title
    let root_title: String = nodes_rows
        .iter()
        .find(|n| n.id == root)
        .map(|n| n.title.clone())
        .unwrap_or_else(|| "Original Document".to_string());

    // Compute sibling order indexes per (parent,label) by created_at
    let mut siblings: HashMap<(Uuid, String), Vec<(Uuid, chrono::DateTime<chrono::Utc>)>> = HashMap::new();
    for r in &edges_ext {
        let label = prompt_by_child
            .get(&r.target_id)
            .and_then(|p| derive_short_label_from_transformation_prompt(p))
            .unwrap_or_else(|| "Transformed".to_string());
        let key = (r.source_id, label);
        let ts = r.child_created_at;
        siblings.entry(key).or_default().push((r.target_id, ts));
    }
    for vec in siblings.values_mut() { 
        vec.sort_by_key(|(_, ts)| *ts); 
    }
    let mut index_by_child: HashMap<Uuid, usize> = HashMap::new();
    for ((_p, _l), vec) in &siblings { 
        for (idx, (child, _)) in vec.iter().enumerate() { 
            index_by_child.insert(*child, idx + 1); 
        } 
    }

    // Build nodes with titles and content previews
    let nodes = nodes_rows
        .into_iter()
        .map(|r| {
            let content_preview = create_content_preview(&r.content, 200);
            
            if r.id == root {
                DocumentGraphNode { 
                    document_id: r.id, 
                    title: r.title, 
                    content_preview,
                    created_at: Some(r.created_at) 
                }
            } else {
                let parent = parent_by_child.get(&r.id).cloned();
                if let Some(_p) = parent {
                    let label = prompt_by_child
                        .get(&r.id)
                        .and_then(|p| derive_short_label_from_transformation_prompt(p))
                        .unwrap_or_else(|| fallback_label_from_prompt(
                            prompt_by_child.get(&r.id).map(|s| s.as_str()).unwrap_or("")
                        ));
                    let idx = index_by_child.get(&r.id).cloned().unwrap_or(1);
                    let count = siblings.get(&(_p, label.clone())).map(|v| v.len()).unwrap_or(1);
                    let mut title = format!("{} - {}", root_title, label);
                    if count > 1 { 
                        title = format!("{} ({})", title, idx); 
                    }
                    DocumentGraphNode { 
                        document_id: r.id, 
                        title, 
                        content_preview,
                        created_at: Some(r.created_at) 
                    }
                } else {
                    // No parent in closure; use document title
                    DocumentGraphNode { 
                        document_id: r.id, 
                        title: r.title, 
                        content_preview,
                        created_at: Some(r.created_at) 
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    Ok(DocumentGraph { 
        nodes, 
        edges: all_edges, 
        root_document_id: root, 
        journey_id: None 
    })
}
