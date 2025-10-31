//! Deletes multiple users in batch with safety checks.
//!
//! This query processes a list of user IDs and attempts to delete each user.
//! It implements safety checks to prevent deletion of admin users and provides
//! detailed success/failure results for each user ID. Designed for admin bulk
//! operations with partial success handling and safety guardrails.

pub struct BatchDeleteUserSuccess {
    pub user_id: uuid::Uuid,
    pub email: String,
}

pub struct BatchDeleteUserFailure {
    pub user_id: uuid::Uuid,
    pub reason: String,
}

pub struct BatchDeleteUsersResult {
    pub success: Vec<BatchDeleteUserSuccess>,
    pub failed: Vec<BatchDeleteUserFailure>,
}

pub async fn batch_delete_users(
    pool: &sqlx::PgPool,
    user_ids: Vec<uuid::Uuid>,
    requesting_admin_id: uuid::Uuid,
) -> anyhow::Result<BatchDeleteUsersResult> {
    let mut success = Vec::new();
    let mut failed = Vec::new();

    for user_id in user_ids {
        if user_id == requesting_admin_id {
            failed.push(BatchDeleteUserFailure {
                user_id,
                reason: "Cannot delete yourself".to_string(),
            });
            continue;
        }

        let user = sqlx::query!(
            r#"SELECT id, email, is_admin FROM users WHERE id = $1"#,
            user_id
        )
        .fetch_optional(pool)
        .await?;

        match user {
            None => {
                failed.push(BatchDeleteUserFailure {
                    user_id,
                    reason: "User not found".to_string(),
                });
                continue;
            }
            Some(user_row) => {
                if user_row.is_admin {
                    failed.push(BatchDeleteUserFailure {
                        user_id,
                        reason: "Cannot delete admin user".to_string(),
                    });
                    continue;
                }

                match sqlx::query!(r#"DELETE FROM users WHERE id = $1"#, user_id)
                    .execute(pool)
                    .await
                {
                    Ok(_) => {
                        success.push(BatchDeleteUserSuccess {
                            user_id,
                            email: user_row.email,
                        });
                    }
                    Err(e) => {
                        failed.push(BatchDeleteUserFailure {
                            user_id,
                            reason: format!("Database error: {}", e),
                        });
                    }
                }
            }
        }
    }

    Ok(BatchDeleteUsersResult { success, failed })
}
