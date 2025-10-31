
//! Lists styles for a user with filtering and enriched details.
//!
//! This function retrieves a paginated list of styles accessible to a user,
//! including styles owned by the user, shared styles, and public styles.
//! Returns enriched data with creator information and access levels.

type StyleWithDetails = crate::db::styles::StyleWithDetails;

#[tracing::instrument(skip(pool))]
#[allow(clippy::too_many_arguments)]
pub async fn list_styles_for_user(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    search_pattern: &str,
    org_ids: &[uuid::Uuid],
    limit: i64,
    offset: i64,
    sort_by: &str,
    sort_order: &str,
    is_favorite_filter: std::option::Option<bool>,
    is_public_filter: std::option::Option<bool>,
) -> std::result::Result<std::vec::Vec<StyleWithDetails>, sqlx::Error> {
    let sql_object_type_style: &str = "style";
    let sql_entity_type_user_str: &str = "user";
    let sql_entity_type_org_str: &str = "organization";
    let org_ids_param = if org_ids.is_empty() {
        &[] as &[uuid::Uuid]
    } else {
        org_ids
    };

    sqlx_conditional_queries::conditional_query_as!(
        StyleWithDetails,
        r#"
        WITH UserOrgShares AS (
            SELECT 
                s.id AS style_id,
                os_user.access_level,
                1 AS priority
            FROM styles s
            JOIN object_shares os_user ON s.id = os_user.object_id
            WHERE os_user.object_type = {sql_object_type_style}
              AND os_user.entity_type::text = {sql_entity_type_user_str}
              AND os_user.entity_id = {user_id}
              AND s.name ILIKE {search_pattern}
            UNION ALL
            SELECT 
                s.id AS style_id,
                os_org.access_level,
                2 AS priority
            FROM styles s
            JOIN object_shares os_org ON s.id = os_org.object_id
            WHERE os_org.object_type = {sql_object_type_style}
              AND os_org.entity_type::text = {sql_entity_type_org_str}
              AND os_org.entity_id = ANY({org_ids_param})
              AND s.name ILIKE {search_pattern}
        ),
        RankedUserOrgShares AS (
            SELECT
                style_id,
                access_level,
                ROW_NUMBER() OVER (PARTITION BY style_id ORDER BY priority ASC, 
                    CASE access_level::text 
                        WHEN 'editor' THEN 1 
                        WHEN 'viewer' THEN 2 
                        ELSE 3 
                    END ASC) as rn
            FROM UserOrgShares
        )
        SELECT 
            s.id as "id", 
            s.user_id as "user_id", 
            s.name as "name", 
            s.html_url as "html_url", 
            s.screenshot_url as "screenshot_url", 
            s.is_public as "is_public",
            s.created_at as "created_at", 
            s.updated_at as "updated_at",
            u.email as "creator_email?",
            CASE
                WHEN s.user_id = {user_id} THEN 'owner'::text
                ELSE rfs.access_level::text 
            END as "current_user_access_level?",
            COALESCE((SELECT EXISTS(SELECT 1 FROM user_favorites WHERE user_id = {user_id} AND entity_id = s.id AND entity_type = 'style')), false) AS is_favorite
        FROM styles s
        LEFT JOIN users u ON s.user_id = u.id 
        LEFT JOIN RankedUserOrgShares rfs ON s.id = rfs.style_id AND rfs.rn = 1
        WHERE 
        (
            CASE 
                WHEN {is_public_filter}::BOOLEAN IS NULL THEN
                    -- Default behavior: return both user's styles and public styles
                    (s.user_id = {user_id} OR s.is_public = true OR rfs.style_id IS NOT NULL)
                WHEN {is_public_filter}::BOOLEAN = true THEN
                    -- Only public styles regardless of user_id
                    s.is_public = true
                WHEN {is_public_filter}::BOOLEAN = false THEN
                    -- User's styles + organization shared styles, but exclude public styles
                    (s.user_id = {user_id} OR rfs.style_id IS NOT NULL) AND s.is_public = false
                ELSE
                    -- Fallback to default behavior
                    (s.user_id = {user_id} OR s.is_public = true OR rfs.style_id IS NOT NULL)
            END
        )
        AND s.name ILIKE {search_pattern}
        AND ({is_favorite_filter}::BOOLEAN IS NULL OR EXISTS(SELECT 1 FROM user_favorites WHERE user_id = {user_id} AND entity_id = s.id AND entity_type = 'style') = {is_favorite_filter})
        ORDER BY {#sort_by_sql_literal} {#sort_order_sql_literal}
        LIMIT {limit} OFFSET {offset}
        "#,
        #user_id = match &user_id { _ => "{user_id}" },
        #search_pattern = match &search_pattern { _ => "{search_pattern}" },
        #sql_object_type_style = match &sql_object_type_style { _ => "{sql_object_type_style}" },
        #sql_entity_type_user_str = match &sql_entity_type_user_str { _ => "{sql_entity_type_user_str}" },
        #sql_entity_type_org_str = match &sql_entity_type_org_str { _ => "{sql_entity_type_org_str}" },
        #org_ids_param = match &org_ids_param { _ => "{org_ids_param}" },
        #limit = match &limit { _ => "{limit}" },
        #offset = match &offset { _ => "{offset}" },
        #is_favorite_filter = match &is_favorite_filter { _ => "{is_favorite_filter}" },
        #is_public_filter = match &is_public_filter { _ => "{is_public_filter}" },
        #sort_by_sql_literal = match sort_by.to_lowercase().as_str() {
            "id" => "s.id",
            "name" => "s.name",
            "created_at" => "s.created_at",
            "updated_at" => "s.updated_at",
            _ => "s.created_at",
        },
        #sort_order_sql_literal = match sort_order.to_lowercase().as_str() {
            "asc" => "ASC",
            "desc" => "DESC",
            _ => "DESC",
        }
    )
    .fetch_all(pool)
    .await
}
