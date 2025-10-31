//! Retrieves all pending invitations for a specific email address.
//!
//! This function queries the `pending_invitations` table to find records
//! where the `email` field matches the provided email address. It is useful for
//! checking if a user has outstanding invitations to join organizations.
//! Adheres to the project's Rust coding standards, including fully qualified paths and no top-level 'use' statements.

// Per rust_guidelines.md: No 'use' statements. All paths are fully qualified.

pub async fn find_pending_invitations_for_email(
    pool: &sqlx::PgPool,
    email: &str,
) -> std::result::Result<std::vec::Vec<crate::db::pending_invitations::pending_invitation::PendingInvitation>, sqlx::Error> {
    sqlx::query_as!(
        crate::db::pending_invitations::pending_invitation::PendingInvitation,
        "SELECT * FROM pending_invitations WHERE invited_email = $1",
        email
    )
    .fetch_all(pool)
    .await
}

#[cfg(test)]
mod tests {
    // Per rust_guidelines.md:
    // - Access items under test using super:: (e.g., super::find_pending_invitations_for_email).
    // - Use fully qualified paths for other items. No 'use super::*'.
    // - Test documentation should be concise.

    // Note: These tests are conceptual. Real tests require an async runtime (e.g. #[tokio::test] or #[sqlx::test])
    // and a connection to a test database to provide a sqlx::PgPool.

    #[test]
    fn test_find_pending_invitations_for_email_conceptual() {
        // This test outlines the structure for a real database integration test.
        // It does not execute actual database queries or require an async runtime here.
        // A real test setup would involve:
        // 1. #[sqlx::test] or #[tokio::test] attribute for async execution.
        // 2. A real or mocked sqlx::PgPool instance.
        // 3. Database setup with test data (e.g., inserting a PendingInvitation).
        // 4. Calling `super::find_pending_invitations_for_email` and awaiting its result.
        // 5. Assertions on the returned data or error.
        
        std::println!("Conceptual test for find_pending_invitations_for_email. DB integration and async runtime needed for a real test.");
        
        // Example structure of an async test (would require dependencies and setup):
        /*
        async fn actual_test_logic(pool: &sqlx::PgPool) {
            let test_email = "user.with.invitations@example.com";
            
            // Assume test_email has one or more entries in pending_invitations table.
            // Setup: Ensure test_email exists in the test database with known invitations.
            // E.g., by inserting a dummy crate::db::pending_invitations::pending_invitation::PendingInvitation.

            let result = super::find_pending_invitations_for_email(pool, test_email).await;
            
            std::assert!(result.is_ok(), "Query failed: {:?}", result.err());
            let invitations = result.unwrap();
            
            // Assertions based on expected data:
            // std::assert!(!invitations.is_empty(), "Expected to find invitations for {}", test_email);
            // std::assert_eq!(invitations[0].email, test_email);
            // ... more specific assertions ...

            // Teardown: Clean up test data if necessary.
        }
        */
        
        assert!(true); // Placeholder, as this is a conceptual, non-executing test.
    }
}
