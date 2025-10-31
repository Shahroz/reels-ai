//! Lists audit logs with pagination and filtering capabilities.
//!
//! Supports filtering by admin user ID, action type, entity type, target entity ID, and date range.
//! Returns paginated results with total count for building efficient admin interfaces.
//! Uses conditional_query_as macro for dynamic filtering with compile-time safety.
//! All filters are optional and can be combined for precise audit trail searches.

pub async fn list_audit_logs(
    pool: &sqlx::PgPool,
    page: i64,
    limit: i64,
    admin_user_id: Option<uuid::Uuid>,
    action_type: Option<&str>,
    target_entity_type: Option<&str>,
    target_entity_id: Option<uuid::Uuid>,
    from_date: Option<chrono::DateTime<chrono::Utc>>,
    to_date: Option<chrono::DateTime<chrono::Utc>>,
) -> anyhow::Result<(Vec<crate::db::audit_logs::AuditLog>, i64)> {
    let offset = (page - 1) * limit;

    // Import types needed by conditional_query_as macro
    use crate::db::audit_logs::AuditLog;
    use crate::sql_utils::count_sql_results::TotalCount;

    // Count query
    let total_count_result = sqlx_conditional_queries::conditional_query_as!(
        TotalCount,
        r#"
        SELECT COUNT(*) as count FROM audit_logs
        WHERE 1=1
        {#admin_user_filter}
        {#action_type_filter}
        {#target_entity_type_filter}
        {#target_entity_id_filter}
        {#from_date_filter}
        {#to_date_filter}
        "#,
        #admin_user_filter = match &admin_user_id {
            Some(_) => "AND admin_user_id = {admin_user_id}",
            None => ""
        },
        #action_type_filter = match &action_type {
            Some(_) => "AND action_type = {action_type}",
            None => ""
        },
        #target_entity_type_filter = match &target_entity_type {
            Some(_) => "AND target_entity_type = {target_entity_type}",
            None => ""
        },
        #target_entity_id_filter = match &target_entity_id {
            Some(_) => "AND target_entity_id = {target_entity_id}",
            None => ""
        },
        #from_date_filter = match &from_date {
            Some(_) => "AND created_at >= {from_date}",
            None => ""
        },
        #to_date_filter = match &to_date {
            Some(_) => "AND created_at <= {to_date}",
            None => ""
        },
        #admin_user_id = match &admin_user_id { _ => "{admin_user_id}" },
        #action_type = match &action_type { _ => "{action_type}" },
        #target_entity_type = match &target_entity_type { _ => "{target_entity_type}" },
        #target_entity_id = match &target_entity_id { _ => "{target_entity_id}" },
        #from_date = match &from_date { _ => "{from_date}" },
        #to_date = match &to_date { _ => "{to_date}" }
    )
    .fetch_one(pool)
    .await?;

    let total_count = total_count_result.count.unwrap_or_default();

    // Data query
    let logs = sqlx_conditional_queries::conditional_query_as!(
        AuditLog,
        r#"
        SELECT
            id, admin_user_id, action_type, target_entity_type, target_entity_id, metadata, created_at
        FROM audit_logs
        WHERE 1=1
        {#admin_user_filter}
        {#action_type_filter}
        {#target_entity_type_filter}
        {#target_entity_id_filter}
        {#from_date_filter}
        {#to_date_filter}
        ORDER BY created_at DESC
        LIMIT {limit}
        OFFSET {offset}
        "#,
        #admin_user_filter = match &admin_user_id {
            Some(_) => "AND admin_user_id = {admin_user_id}",
            None => ""
        },
        #action_type_filter = match &action_type {
            Some(_) => "AND action_type = {action_type}",
            None => ""
        },
        #target_entity_type_filter = match &target_entity_type {
            Some(_) => "AND target_entity_type = {target_entity_type}",
            None => ""
        },
        #target_entity_id_filter = match &target_entity_id {
            Some(_) => "AND target_entity_id = {target_entity_id}",
            None => ""
        },
        #from_date_filter = match &from_date {
            Some(_) => "AND created_at >= {from_date}",
            None => ""
        },
        #to_date_filter = match &to_date {
            Some(_) => "AND created_at <= {to_date}",
            None => ""
        },
        #admin_user_id = match &admin_user_id { _ => "{admin_user_id}" },
        #action_type = match &action_type { _ => "{action_type}" },
        #target_entity_type = match &target_entity_type { _ => "{target_entity_type}" },
        #target_entity_id = match &target_entity_id { _ => "{target_entity_id}" },
        #from_date = match &from_date { _ => "{from_date}" },
        #to_date = match &to_date { _ => "{to_date}" },
        #limit = match &limit { _ => "{limit}" },
        #offset = match &offset { _ => "{offset}" }
    )
    .fetch_all(pool)
    .await?;

    Ok((logs, total_count))
}

