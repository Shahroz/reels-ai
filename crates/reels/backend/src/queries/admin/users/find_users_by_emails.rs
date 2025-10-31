//! Finds multiple users by their email addresses in batch.
//!
//! This helper query looks up users by a list of email addresses and returns
//! those that exist. Useful for admin operations that need to validate and resolve
//! email addresses to user IDs before performing bulk operations.

pub struct EmailUserMapping {
    pub email: String,
    pub user_id: uuid::Uuid,
}

pub async fn find_users_by_emails(
    pool: &sqlx::PgPool,
    emails: Vec<String>,
) -> anyhow::Result<Vec<EmailUserMapping>> {
    if emails.is_empty() {
        return Ok(Vec::new());
    }

    let emails_lower: Vec<String> = emails.iter().map(|e| e.to_lowercase()).collect();

    let users = sqlx::query!(
        r#"
        SELECT id, email
        FROM users
        WHERE LOWER(email) = ANY($1)
        "#,
        &emails_lower[..]
    )
    .fetch_all(pool)
    .await?;

    Ok(users
        .into_iter()
        .map(|row| EmailUserMapping {
            email: row.email,
            user_id: row.id,
        })
        .collect())
}
