//! Finds all pending invitations sent out by a specific organization.
//!
//! This function retrieves all invitations that have been sent by a
//! given organization, intended for listing outstanding invitations
//! from an organization's perspective.
//! The results are ordered by creation date.

pub async fn find_pending_invitations_for_organization(
    pool: &sqlx::postgres::PgPool,
    organization_id_filter: sqlx::types::Uuid,
) -> Result<Vec<crate::db::pending_invitations::SentInvitationDbRow>, sqlx::Error> {
    sqlx::query_as!(
        crate::db::pending_invitations::SentInvitationDbRow,
        r#"
        SELECT
            id,
            organization_id,
            invited_email,
            role_to_assign,
            invited_by_user_id,
            created_at,
            token_expires_at
        FROM pending_invitations
        WHERE organization_id = $1
        ORDER BY created_at DESC
        "#,
        organization_id_filter
    )
    .fetch_all(pool)
    .await
}