//! Processes batch member additions to an organization.
//!
//! This query attempts to add multiple users to an organization by email.
//! It validates each email, checks if the user exists, verifies they're not already
//! a member, and returns detailed success/failure results for each email.
//! Designed for admin bulk operations with partial success handling.

pub struct BatchAddMemberSuccess {
    pub email: String,
    pub user_id: uuid::Uuid,
    pub member: crate::db::organization_members::OrganizationMember,
}

pub struct BatchAddMemberFailure {
    pub email: String,
    pub reason: String,
}

pub struct BatchAddMembersResult {
    pub success: Vec<BatchAddMemberSuccess>,
    pub failed: Vec<BatchAddMemberFailure>,
}

pub async fn batch_add_members(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    organization_id: uuid::Uuid,
    emails: Vec<String>,
    role: &str,
    invited_by_user_id: uuid::Uuid,
) -> anyhow::Result<BatchAddMembersResult> {
    let mut success = Vec::new();
    let mut failed = Vec::new();

    for email in emails {
        let email_lower = email.to_lowercase();

        let user_result = sqlx::query!(
            r#"SELECT id, email FROM users WHERE LOWER(email) = $1"#,
            email_lower
        )
        .fetch_optional(&mut **tx)
        .await?;

        match user_result {
            None => {
                failed.push(BatchAddMemberFailure {
                    email,
                    reason: "User not found".to_string(),
                });
                continue;
            }
            Some(user) => {
                let existing_member = sqlx::query!(
                    r#"
                    SELECT user_id 
                    FROM organization_members 
                    WHERE organization_id = $1 AND user_id = $2
                    "#,
                    organization_id,
                    user.id
                )
                .fetch_optional(&mut **tx)
                .await?;

                if existing_member.is_some() {
                    failed.push(BatchAddMemberFailure {
                        email,
                        reason: "User is already a member of this organization".to_string(),
                    });
                    continue;
                }

                match crate::queries::organizations::add_member::add_member(
                    tx,
                    organization_id,
                    user.id,
                    role,
                    "active",
                    Some(invited_by_user_id),
                )
                .await
                {
                    Ok(member) => {
                        success.push(BatchAddMemberSuccess {
                            email: user.email,
                            user_id: user.id,
                            member,
                        });
                    }
                    Err(e) => {
                        failed.push(BatchAddMemberFailure {
                            email,
                            reason: format!("Failed to add member: {}", e),
                        });
                    }
                }
            }
        }
    }

    Ok(BatchAddMembersResult { success, failed })
}
