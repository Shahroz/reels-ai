use sqlx::types::Uuid;
use crate::queries::assets::lineage::types::{StudioGraph, StudioGraphEdge, StudioGraphNode, DerivationParams};

fn derive_short_label_from_prompt(prompt: &str) -> Option<String> {
    let p = prompt.to_lowercase();
    let contains = |needle: &str| p.contains(needle);

    if contains("golden-hour") || contains("day to dusk") { return Some("Day to Dusk".to_string()); }
    if contains("virtually clean") || contains("tidy") || contains("clutter") { return Some("Virtual Cleaning".to_string()); }
    if contains("minimal neutral décor") || contains("soft styling") { return Some("Soft Styling".to_string()); }
    if contains("fix lens distortion") || contains("straighten verticals") { return Some("Straighten Verticals".to_string()); }
    if contains("lift exposure") || contains("dynamic range") { return Some("Balanced Exposure".to_string()); }
    if contains("remove color casts") || contains("white balance") { return Some("Neutral White Balance".to_string()); }
    if contains("restore exterior window view") { return Some("Window View".to_string()); }
    if contains("replace overcast sky") { return Some("Sky: Subtle Blue".to_string()); }
    if contains("green up grass") { return Some("Lawn Refresh".to_string()); }
    if contains("clean driveway") || contains("oil spots") { return Some("Driveway Clean".to_string()); }
    if contains("remove personal items") { return Some("Remove Personal Items".to_string()); }
    if contains("repair wall scuffs") { return Some("Wall Touch-Up".to_string()); }
    if contains("turn on interior lights") { return Some("Lights On (Warm)".to_string()); }
    if contains("reduce distracting reflections") { return Some("Cut Reflections".to_string()); }
    if contains("replace visible tv") || contains("monitor screens") { return Some("Screens to Black".to_string()); }
    if contains("remove rain") || contains("puddles") { return Some("Rain → Dry".to_string()); }
    if contains("reduce noise") && contains("detail") { return Some("Clean + Detail".to_string()); }

    // Virtual stage/renovate/paint patterns
    if let Some(style) = p.strip_prefix("virtually stage in ").and_then(|s| s.split(" style").next()) {
        let s_cap = style.trim().split_whitespace().map(|w| {
            let mut c = w.chars();
            match c.next() { Some(f) => f.to_uppercase().collect::<String>() + c.as_str(), None => String::new() }
        }).collect::<Vec<_>>().join(" ");
        return Some(format!("Stage: {}", s_cap));
    }
    if let Some(style) = p.strip_prefix("virtually renovate to ").and_then(|s| s.split(" style").next()) {
        let s_cap = style.trim().split_whitespace().map(|w| {
            let mut c = w.chars();
            match c.next() { Some(f) => f.to_uppercase().collect::<String>() + c.as_str(), None => String::new() }
        }).collect::<Vec<_>>().join(" ");
        return Some(format!("Renovate: {}", s_cap));
    }
    if let Some(color) = p.strip_prefix("change wall paint to ").and_then(|s| s.split(';').next()) {
        let s_cap = color.trim().split_whitespace().map(|w| {
            let mut c = w.chars();
            match c.next() { Some(f) => f.to_uppercase().collect::<String>() + c.as_str(), None => String::new() }
        }).collect::<Vec<_>>().join(" ");
        return Some(format!("Paint: {}", s_cap));
    }
    None
}

fn fallback_label_from_prompt(prompt: &str) -> String {
    let s = prompt.trim();
    if s.is_empty() { return "Derived".to_string(); }
    let max_len = 28usize;
    let mut out = s.chars().take(max_len).collect::<String>();
    if s.len() > out.len() { out.push_str("…"); }
    // Capitalize first letter
    if let Some(first) = out.get(0..1) {
        out.replace_range(0..1, &first.to_uppercase().to_string());
    }
    out
}

pub async fn get_graph_for_asset(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    asset_id: Uuid,
) -> Result<StudioGraph, sqlx::Error> {
    // Fetch user's organization IDs for permission checking
    let org_ids: Vec<Uuid> = sqlx::query_scalar!(
        "SELECT organization_id FROM organization_members WHERE user_id = $1 AND status = 'active'",
        user_id
    )
    .fetch_all(pool)
    .await?;

    // Ensure the asset belongs to the user, is public, or is shared with user/organization (including collection-level shares)
    let access_check = sqlx::query!(
        r#"
        SELECT a.user_id, a.is_public, 
               COALESCE(os_user.access_level, os_org.access_level, cs_user.access_level, cs_org.access_level)::TEXT as shared_access_level
        FROM assets a
        LEFT JOIN object_shares os_user ON a.id = os_user.object_id 
            AND os_user.object_type = 'asset' 
            AND os_user.entity_type = 'user' 
            AND os_user.entity_id = $2
        LEFT JOIN object_shares os_org ON a.id = os_org.object_id 
            AND os_org.object_type = 'asset' 
            AND os_org.entity_type = 'organization' 
            AND os_org.entity_id = ANY($3)
        LEFT JOIN object_shares cs_user ON a.collection_id = cs_user.object_id 
            AND cs_user.object_type = 'collection' 
            AND cs_user.entity_type = 'user' 
            AND cs_user.entity_id = $2
        LEFT JOIN object_shares cs_org ON a.collection_id = cs_org.object_id 
            AND cs_org.object_type = 'collection' 
            AND cs_org.entity_type = 'organization' 
            AND cs_org.entity_id = ANY($3)
        WHERE a.id = $1
        "#,
        asset_id,
        user_id,
        &org_ids[..]
    )
    .fetch_one(pool)
    .await?;

    // Check if user has access (owner, public asset, direct shares, or collection-level shares)
    let has_access = access_check.user_id == Some(user_id) || 
                     access_check.is_public || 
                     access_check.shared_access_level.is_some();
    
    if !has_access { 
        return Err(sqlx::Error::RowNotFound); 
    }

    // Use recursive CTE to get full closure (ancestors+descendants) from the seed
    // Only include derived assets that belong to the requesting user
    let all_ids_rows = sqlx::query!(
        r#"
        WITH RECURSIVE ancestors(id) AS (
            SELECT source_id FROM provenance_edges WHERE target_type = 'asset' AND source_type = 'asset' AND target_id = $1
            UNION
            SELECT d.source_id FROM provenance_edges d JOIN ancestors a ON d.target_id = a.id
             WHERE d.target_type = 'asset' AND d.source_type = 'asset'
        ),
        descendants(id) AS (
            SELECT target_id FROM provenance_edges WHERE source_type = 'asset' AND target_type = 'asset' AND source_id = $1
            UNION
            SELECT d.target_id FROM provenance_edges d JOIN descendants c ON d.source_id = c.id
             WHERE d.source_type = 'asset' AND d.target_type = 'asset'
        ),
        all_ids(id) AS (
            SELECT $1::uuid
            UNION SELECT id FROM ancestors
            UNION SELECT id FROM descendants
        )
        SELECT id FROM all_ids
        "#,
        asset_id
    )
    .fetch_all(pool)
    .await?;
    let mut asset_ids: Vec<Uuid> = all_ids_rows.into_iter().filter_map(|r| r.id).collect();
    if !asset_ids.contains(&asset_id) { asset_ids.push(asset_id); }

    // Identify a provisional root from the first closure
    let edges_seed = sqlx::query!(
        r#"SELECT source_id, target_id FROM provenance_edges
            WHERE (source_type='asset' AND target_type='asset')
              AND (source_id = ANY($1) OR target_id = ANY($1))"#,
        &asset_ids[..]
    )
    .fetch_all(pool)
    .await?;

    let mut root = asset_id;
    for r in &asset_ids {
        let has_incoming = edges_seed.iter().any(|e| e.target_id == *r);
        if !has_incoming { root = *r; break; }
    }

    // Recompute closure strictly as all descendants from the root (for complete siblings view)
    // Include assets that the user owns OR has shared access to (including collection-level shares)
    let all_desc_rows = sqlx::query!(
        r#"
        WITH RECURSIVE descendants(id) AS (
            SELECT $1::uuid
            UNION
            SELECT d.target_id FROM provenance_edges d 
            JOIN descendants c ON d.source_id = c.id
            JOIN assets a ON a.id = d.target_id
            LEFT JOIN object_shares os_user ON a.id = os_user.object_id 
                AND os_user.object_type = 'asset' 
                AND os_user.entity_type = 'user' 
                AND os_user.entity_id = $2
            LEFT JOIN object_shares os_org ON a.id = os_org.object_id 
                AND os_org.object_type = 'asset' 
                AND os_org.entity_type = 'organization' 
                AND os_org.entity_id = ANY($3)
            LEFT JOIN object_shares cs_user ON a.collection_id = cs_user.object_id 
                AND cs_user.object_type = 'collection' 
                AND cs_user.entity_type = 'user' 
                AND cs_user.entity_id = $2
            LEFT JOIN object_shares cs_org ON a.collection_id = cs_org.object_id 
                AND cs_org.object_type = 'collection' 
                AND cs_org.entity_type = 'organization' 
                AND cs_org.entity_id = ANY($3)
             WHERE d.source_type='asset' AND d.target_type='asset'
               AND (a.user_id = $2 OR a.is_public OR os_user.id IS NOT NULL OR os_org.id IS NOT NULL OR cs_user.id IS NOT NULL OR cs_org.id IS NOT NULL)
        )
        SELECT id FROM descendants
        "#,
        root,
        user_id,
        &org_ids[..]
    )
    .fetch_all(pool)
    .await?;
    let asset_ids: Vec<Uuid> = all_desc_rows.into_iter().filter_map(|r| r.id).collect();

    // Fetch edges with params and child created_at within the closure
    // Include edges where the target asset is owned by or shared with the requesting user (including collection-level shares)
    let edges_ext = sqlx::query!(
        r#"SELECT d.source_id, d.target_id, d.params, child.created_at as child_created_at
            FROM provenance_edges d
            JOIN assets child ON child.id = d.target_id
            LEFT JOIN object_shares os_user ON child.id = os_user.object_id 
                AND os_user.object_type = 'asset' 
                AND os_user.entity_type = 'user' 
                AND os_user.entity_id = $2
            LEFT JOIN object_shares os_org ON child.id = os_org.object_id 
                AND os_org.object_type = 'asset' 
                AND os_org.entity_type = 'organization' 
                AND os_org.entity_id = ANY($3)
            LEFT JOIN object_shares cs_user ON child.collection_id = cs_user.object_id 
                AND cs_user.object_type = 'collection' 
                AND cs_user.entity_type = 'user' 
                AND cs_user.entity_id = $2
            LEFT JOIN object_shares cs_org ON child.collection_id = cs_org.object_id 
                AND cs_org.object_type = 'collection' 
                AND cs_org.entity_type = 'organization' 
                AND cs_org.entity_id = ANY($3)
            WHERE d.source_type='asset' AND d.target_type='asset'
              AND (d.source_id = ANY($1) OR d.target_id = ANY($1))
              AND (child.user_id = $2 OR child.is_public OR os_user.id IS NOT NULL OR os_org.id IS NOT NULL OR cs_user.id IS NOT NULL OR cs_org.id IS NOT NULL)"#,
        &asset_ids[..],
        user_id,
        &org_ids[..]
    )
    .fetch_all(pool)
    .await?;

    let all_edges: Vec<StudioGraphEdge> = edges_ext
        .iter()
        .map(|r| StudioGraphEdge { source_asset_id: r.source_id, derived_asset_id: r.target_id })
        .collect();

    // Fetch node details - include assets that the user owns or has shared access to (including collection-level shares)
    let nodes_rows = sqlx::query!(
        r#"SELECT a.id, a.url, a.name, a.created_at FROM assets a
           LEFT JOIN object_shares os_user ON a.id = os_user.object_id 
               AND os_user.object_type = 'asset' 
               AND os_user.entity_type = 'user' 
               AND os_user.entity_id = $3
           LEFT JOIN object_shares os_org ON a.id = os_org.object_id 
               AND os_org.object_type = 'asset' 
               AND os_org.entity_type = 'organization' 
               AND os_org.entity_id = ANY($4)
           LEFT JOIN object_shares cs_user ON a.collection_id = cs_user.object_id 
               AND cs_user.object_type = 'collection' 
               AND cs_user.entity_type = 'user' 
               AND cs_user.entity_id = $3
           LEFT JOIN object_shares cs_org ON a.collection_id = cs_org.object_id 
               AND cs_org.object_type = 'collection' 
               AND cs_org.entity_type = 'organization' 
               AND cs_org.entity_id = ANY($4)
           WHERE a.id = ANY($1) 
           AND (a.id = $2 OR a.user_id = $3 OR a.is_public OR os_user.id IS NOT NULL OR os_org.id IS NOT NULL OR cs_user.id IS NOT NULL OR cs_org.id IS NOT NULL)"#,
        &asset_ids[..],
        root,
        user_id,
        &org_ids[..]
    )
    .fetch_all(pool)
    .await?;

    // Root computed above

    // Build parent links and raw retouch prompts for label derivation
    use std::collections::HashMap;
    let mut parent_by_child: HashMap<Uuid, Uuid> = HashMap::new();
    let mut prompt_by_child: HashMap<Uuid, String> = HashMap::new();
    for r in &edges_ext {
        parent_by_child.insert(r.target_id, r.source_id);
        let params = &r.params;
        // Prefer typed enum; fallback to legacy direct RetouchParams if present
        if let Ok(enum_params) = serde_json::from_value::<DerivationParams>(params.clone()) {
            if let DerivationParams::Retouch(retouch) = enum_params {
                prompt_by_child.insert(r.target_id, retouch.retouch_prompt);
                continue;
            }
        }
        if let Ok(legacy) = serde_json::from_value::<crate::queries::assets::lineage::types::RetouchParams>(params.clone()) {
            prompt_by_child.insert(r.target_id, legacy.retouch_prompt);
        }
    }

    // Get root name
    let root_name: String = nodes_rows.iter().find(|n| n.id == root).map(|n| n.name.clone()).unwrap_or_else(|| "Original".to_string());

    // Compute sibling order indexes per (parent,label) by created_at
    let mut siblings: HashMap<(Uuid, String), Vec<(Uuid, chrono::DateTime<chrono::Utc>)>> = HashMap::new();
    for r in &edges_ext {
        let label = prompt_by_child
            .get(&r.target_id)
            .and_then(|p| derive_short_label_from_prompt(p))
            .unwrap_or_else(|| "Derived".to_string());
        let key = (r.source_id, label);
        let ts = r.child_created_at;
        siblings.entry(key).or_default().push((r.target_id, ts));
    }
    for vec in siblings.values_mut() { vec.sort_by_key(|(_, ts)| *ts); }
    let mut index_by_child: HashMap<Uuid, usize> = HashMap::new();
    for ((_p, _l), vec) in &siblings { for (idx, (child, _)) in vec.iter().enumerate() { index_by_child.insert(*child, idx + 1); } }

    // Build nodes with names
    let nodes = nodes_rows
        .into_iter()
        .map(|r| {
            if r.id == root {
                StudioGraphNode { asset_id: r.id, url: r.url, name: r.name, created_at: Some(r.created_at) }
            } else {
                let parent = parent_by_child.get(&r.id).cloned();
                if let Some(_p) = parent {
                    let label = prompt_by_child
                        .get(&r.id)
                        .and_then(|p| derive_short_label_from_prompt(p))
                        .unwrap_or_else(|| fallback_label_from_prompt(prompt_by_child.get(&r.id).map(|s| s.as_str()).unwrap_or("")));
                    let idx = index_by_child.get(&r.id).cloned().unwrap_or(1);
                    let count = siblings.get(&(_p, label.clone())).map(|v| v.len()).unwrap_or(1);
                    let mut name = format!("{} - {}", root_name, label);
                    if count > 1 { name = format!("{} ({})", name, idx); }
                    StudioGraphNode { asset_id: r.id, url: r.url, name, created_at: Some(r.created_at) }
                } else {
                    // No parent in closure; use asset name
                    StudioGraphNode { asset_id: r.id, url: r.url, name: r.name, created_at: Some(r.created_at) }
                }
            }
        })
        .collect::<Vec<_>>();

    Ok(StudioGraph { nodes, edges: all_edges, root_asset_id: root, journey_id: None })
}

