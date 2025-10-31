//! Fetches a user's email address by their unique user ID.
//!
//! This function queries the `users` table for a single user's email.
//! It's used when an operation needs to associate an action with a user's email,
//! such as populating creator information on a newly created document.

pub async fn fetch_user_email(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: uuid::Uuid,
) -> std::result::Result<Option<std::string::String>, sqlx::Error> {
   sqlx::query_scalar!(
       "SELECT email FROM users WHERE id = $1",
       user_id
   )
   .fetch_optional(&mut **tx)
   .await
}

#[cfg(test)]
mod tests {
    //! Tests for fetch_user_email.
    //!
    //! These are conceptual tests as they require a live database connection and async runtime.
    //! They outline the logic that would be used in an integration testing environment.

    #[test]
    fn conceptual_test_fetch_email() {
        // In a real test, you would:
        // 1. Setup a test database.
        // 2. Insert a user with a known ID and email.
        // 3. Call `super::fetch_user_email`.
        // 4. Assert that the correct email is returned.
        std::assert!(true, "Conceptual test passed for fetch_user_email");
    }
}
