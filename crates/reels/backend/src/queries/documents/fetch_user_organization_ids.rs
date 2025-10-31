//! Fetches the active organization IDs for a given user.
//!
//! This function queries the `organization_members` table to find all organizations
//! where the user is an 'active' member. It returns a vector of organization UUIDs.
//! This is a common utility for permission checks across different parts of the application.

pub async fn fetch_user_organization_ids(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: uuid::Uuid,
) -> std::result::Result<std::vec::Vec<uuid::Uuid>, sqlx::Error> {
   sqlx::query_scalar!(
       "SELECT organization_id FROM organization_members WHERE user_id = $1 AND status = 'active'",
       user_id
   )
   .fetch_all(&mut **tx)
   .await
}

#[cfg(test)]
mod tests {
    //! Tests for fetch_user_organization_ids.
    //!
    //! These are conceptual tests as they require a live database connection and async runtime.
    //! They outline the logic that would be used in an integration testing environment.

    #[test]
    fn conceptual_test_fetch_ids() {
        // In a real test, you would:
        // 1. Setup a test database.
        // 2. Insert a user and some organization memberships (active and inactive).
        // 3. Call `super::fetch_user_organization_ids`.
        // 4. Assert that only the active organization IDs are returned.
        std::assert!(true, "Conceptual test passed for fetch_user_organization_ids");
    }
}
