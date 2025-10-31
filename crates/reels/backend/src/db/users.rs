// backend/src/db/users.rs
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{postgres::PgPool, types::Uuid, Error, FromRow};
use sqlx_conditional_queries::conditional_query_as;
use tracing::instrument;
use utoipa::ToSchema;

// Define the User struct matching the database table
// Ensure fields match the columns in your 'users' table
#[derive(Debug, FromRow, Clone, Serialize, ToSchema)]
pub struct User {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type=String)]
    pub id: Uuid, // Assuming UUID in DB
    pub email: String,
    pub password_hash: Option<String>, // NULL for OAuth-only users
    pub stripe_customer_id: Option<String>,
    pub email_verified: bool,
    pub is_admin: bool, // Added is_admin field
    pub status: String,
    pub feature_flags: Vec<String>,
    #[schema(value_type = String, example = "2024-04-21T10:00:00Z")]
    pub created_at: DateTime<Utc>, // Assuming TIMESTAMPTZ
    #[schema(value_type = String, example = "2024-04-21T10:00:00Z")]
    pub updated_at: DateTime<Utc>, // Assuming TIMESTAMPTZ
    // Add verification_token and token_expiry if needed here, matching migration
    pub verification_token: Option<String>,
    #[schema(value_type = Option<String>,  example = "2024-04-21T10:00:00Z")]
    pub token_expiry: Option<DateTime<Utc>>,
    // Trial-related fields
    #[schema(value_type = Option<String>,  example = "2024-04-21T10:00:00Z")]
    pub trial_started_at: Option<DateTime<Utc>>,
    #[schema(value_type = Option<String>,  example = "2024-04-21T10:00:00Z")]
    pub trial_ended_at: Option<DateTime<Utc>>,
    pub subscription_status: Option<String>,
    // Magic link token version for single-use enforcement
    pub token_version: i32,
}

#[derive(Debug, Serialize, serde::Deserialize, ToSchema)]
pub struct PublicUser {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000", format = "uuid", value_type=String)]
    pub id: Uuid,
    pub email: String,
    pub stripe_customer_id: Option<String>,
    pub email_verified: bool,
    pub is_admin: bool, // Added is_admin field
    pub status: String,
    pub feature_flags: Vec<String>,
    #[schema(value_type = String,  example = "2024-04-21T10:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String,  example = "2024-04-21T10:00:00Z")]
    pub updated_at: DateTime<Utc>,
    pub verification_token: Option<String>,
    #[schema(value_type = Option<String>,  example = "2024-04-21T10:00:00Z")]
    pub token_expiry: Option<DateTime<Utc>>,
    // Trial-related fields
    #[schema(value_type = Option<String>,  example = "2024-04-21T10:00:00Z")]
    pub trial_started_at: Option<DateTime<Utc>>,
    #[schema(value_type = Option<String>,  example = "2024-04-21T10:00:00Z")]
    pub trial_ended_at: Option<DateTime<Utc>>,
    pub subscription_status: Option<String>,
    // Magic link token version for single-use enforcement (not exposed in API)
    pub token_version: i32,
}

impl From<User> for PublicUser {
    fn from(user: User) -> Self {
        PublicUser {
            id: user.id,
            email: user.email,
            stripe_customer_id: user.stripe_customer_id,
            email_verified: user.email_verified,
            is_admin: user.is_admin, // Copy is_admin field
            status: user.status,
            feature_flags: user.feature_flags,
            created_at: user.created_at,
            updated_at: user.updated_at,
            verification_token: user.verification_token,
            token_expiry: user.token_expiry,
            trial_started_at: user.trial_started_at,
            trial_ended_at: user.trial_ended_at,
            subscription_status: user.subscription_status,
            token_version: user.token_version,
        }
    }
}

/// Creates a new user in the database.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `email` - The user's email address.
/// * `password_hash` - The user's hashed password.
///
/// # Returns
///
/// A `Result` containing the new user's UUID on success, or an `sqlx::Error` on failure.
#[instrument(skip(pool, password_hash))]
pub async fn create_user(pool: &PgPool, email: &str, password_hash: &str) -> Result<Uuid, Error> {
    let email_lower = email.to_lowercase();
    let result = sqlx::query!(
        r#"
        INSERT INTO users (email, password_hash)
        VALUES ($1, $2)
        RETURNING id AS "id: uuid::Uuid"
        "#,
        email_lower,
        password_hash
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(record) => Ok(record.id),
        Err(e) => {
            eprintln!("Failed to create user: {e}"); // Basic logging
            Err(e)
        }
    }
}

/// Finds a user by their email address.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `email` - The email address to search for.
///
/// # Returns
///
/// A `Result` containing an `Option<User>` on success, or an `sqlx::Error` on failure.
/// The `Option` is `Some(User)` if found, `None` otherwise.
#[instrument(skip(pool))]
pub async fn find_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, Error> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT
            id AS "id: uuid::Uuid",
            email,
            password_hash,
            stripe_customer_id,
            email_verified,
            status,
            feature_flags,
            is_admin,
            created_at,
            updated_at,
            verification_token,
            token_expiry,
            trial_started_at,
            trial_ended_at,
            subscription_status,
            token_version
        FROM users
        WHERE email ILIKE $1
        "#,
        email
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

/// Finds a user by their ID.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `user_id` - The UUID of the user to find.
///
/// # Returns
///
/// A `Result` containing an `Option<User>` on success, or an `sqlx::Error` on failure.
/// The `Option` is `Some(User)` if found, `None` otherwise.
#[instrument(skip(pool))]
pub async fn find_user_by_id(pool: &PgPool, user_id: Uuid) -> Result<Option<User>, Error> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT
            id AS "id: uuid::Uuid",
            email,
            password_hash,
            stripe_customer_id,
            email_verified,
            status,
            feature_flags,
            is_admin,
            created_at,
            updated_at,
            verification_token,
            token_expiry,
            trial_started_at,
            trial_ended_at,
            subscription_status,
            token_version
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

/// Updates the password hash for a given user.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `user_id` - The UUID of the user whose password should be updated.
/// * `new_password_hash` - The new hashed password.
///
/// # Returns
///
/// A `Result` indicating success or an `sqlx::Error` on failure.
/// Returns `sqlx::Error::RowNotFound` if the `user_id` does not exist.
#[instrument(skip(pool, new_password_hash))]
pub async fn update_user_password_hash(
    pool: &PgPool,
    user_id: Uuid,
    new_password_hash: &str,
) -> Result<(), Error> {
    let result = sqlx::query!(
        r#"
        UPDATE users
        SET password_hash = $1, updated_at = NOW()
        WHERE id = $2::uuid
        "#,
        new_password_hash,
        user_id
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        Err(sqlx::Error::RowNotFound)
    } else {
        Ok(())
    }
}

/// Lists users with pagination, sorting, and filtering.
#[instrument(skip(pool))]
pub async fn list_users(
    pool: &PgPool,
    page: i64,
    limit: i64,
    sort_by: &str,
    sort_order: &str,
    search_query: Option<&str>,
    status_filter: Option<&str>,
) -> Result<(Vec<User>, i64), Error> {
    let offset = (page - 1) * limit;
    let search_pattern = search_query.map(|s| format!("%{}%", s.to_lowercase()));
    let status_filter_value = status_filter;

    // Count query using TotalCount struct pattern from usage_statistics
    type TotalCount = crate::sql_utils::count_sql_results::TotalCount;
    let total_count_result = conditional_query_as!(
        TotalCount,
        r#"
        SELECT count(*) as count FROM users
        WHERE 1=1
        {#search_filter}
        {#status_filter}
        "#,
        #search_filter = match &search_pattern {
            Some(_) => "AND (LOWER(email) LIKE {search_pattern} OR LOWER(stripe_customer_id) LIKE {search_pattern})",
            None => ""
        },
        #status_filter = match status_filter {
            Some(_) => "AND status = {status_filter_value}",
            None => ""
        },
        #search_pattern = match &search_pattern { _ => "{search_pattern}" },
        #status_filter_value = match &status_filter { _ => "{status_filter_value}" }
    )
        .fetch_one(pool)
        .await?;
    
    let total_count = total_count_result.count.unwrap_or_default();

    // Data query
    let users = conditional_query_as!(
        User,
        r#"
        SELECT
            id, email, password_hash, stripe_customer_id,
            email_verified, status, feature_flags, is_admin, created_at, updated_at,
            verification_token, token_expiry, trial_started_at, trial_ended_at, subscription_status, token_version
        FROM users
        WHERE 1=1
        {#search_filter}
        {#status_filter}
        ORDER BY {#sort_column} {#sort_direction}
        LIMIT {limit} OFFSET {offset}
        "#,
        #search_filter = match &search_pattern {
            Some(_) => "AND (LOWER(email) LIKE {search_pattern} OR LOWER(stripe_customer_id) LIKE {search_pattern})",
            None => ""
        },
        #status_filter = match status_filter {
            Some(_) => "AND status = {status_filter_value}",
            None => ""
        },
        #search_pattern = match &search_pattern { _ => "{search_pattern}" },
        #status_filter_value = match &status_filter { _ => "{status_filter_value}" },
        #limit = match &limit { _ => "{limit}" },
        #offset = match &offset { _ => "{offset}" },
        #sort_column = match sort_by {
            "email" => "email",
            "status" => "status", 
            "updated_at" => "updated_at",
            _ => "created_at",
        },
        #sort_direction = match sort_order.to_lowercase().as_str() {
            "asc" => "ASC",
            _ => "DESC"
        },
    )
    .fetch_all(pool)
    .await?;

    Ok((users, total_count))
}

/// Creates a new user by an administrator.
#[instrument(skip(pool, password_hash))]
pub async fn admin_create_user(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
    is_admin: bool,
    status: &str,
    feature_flags: &[String],
) -> Result<User, Error> {
    let email_lower = email.to_lowercase();
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (email, password_hash, is_admin, status, feature_flags)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING
            id AS "id: uuid::Uuid", email, password_hash, stripe_customer_id,
            email_verified, status, feature_flags, is_admin, created_at, updated_at,
            verification_token, token_expiry, trial_started_at, trial_ended_at, subscription_status, token_version
        "#,
        email_lower,
        password_hash,
        is_admin,
        status,
        feature_flags
    )
    .fetch_one(pool)
    .await?;
    Ok(user)
}

/// Updates a user by an administrator.
#[instrument(skip(pool))]
pub async fn admin_update_user(
    pool: &PgPool,
    user_id: Uuid,
    is_admin: bool,
    status: &str,
    feature_flags: &[String],
) -> Result<User, Error> {
    let user = sqlx::query_as!(
        User,
        r#"
        UPDATE users
        SET is_admin = $1, status = $2, feature_flags = $3, updated_at = NOW()
        WHERE id = $4
        RETURNING
            id AS "id: uuid::Uuid", email, password_hash, stripe_customer_id,
            email_verified, status, feature_flags, is_admin, created_at, updated_at,
            verification_token, token_expiry, trial_started_at, trial_ended_at, subscription_status, token_version
        "#,
        is_admin,
        status,
        feature_flags,
        user_id
    )
    .fetch_one(pool)
    .await?;
    Ok(user)
}

/// Sets the status for a given user.
#[instrument(skip(pool))]
pub async fn set_user_status(pool: &PgPool, user_id: Uuid, status: &str) -> Result<(), Error> {
    let result = sqlx::query!(
        r#"
        UPDATE users
        SET status = $1, updated_at = NOW()
        WHERE id = $2
        "#,
        status,
        user_id
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        Err(sqlx::Error::RowNotFound)
    } else {
        Ok(())
    }
}

/// Sets the verification token and expiry for a user.
#[instrument(skip(pool, token))]
pub async fn set_user_verification_token(
    pool: &PgPool,
    user_id: Uuid,
    token: &str,
    expires_at: chrono::DateTime<chrono::Utc>,
) -> Result<(), Error> {
    let result = sqlx::query!(
        r#"
        UPDATE users
        SET verification_token = $1, token_expiry = $2, updated_at = NOW()
        WHERE id = $3
        "#,
        token,
        expires_at,
        user_id
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        Err(sqlx::Error::RowNotFound)
    } else {
        Ok(())
    }
}

/// Updates the Stripe customer ID for a given user.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `user_id` - The UUID of the user to update.
/// * `stripe_customer_id` - The Stripe customer ID to store.
///
/// # Returns
///
/// A `Result` indicating success or an `sqlx::Error` on failure.
/// Returns `sqlx::Error::RowNotFound` if the `user_id` does not exist.
#[instrument(skip(pool, stripe_customer_id))]
pub async fn update_user_stripe_id(
    pool: &PgPool,
    user_id: Uuid,
    stripe_customer_id: &str,
) -> Result<(), Error> {
    let result = sqlx::query!(
        r#"
        UPDATE users
        SET stripe_customer_id = $1, updated_at = NOW()
        WHERE id = $2::uuid
        "#,
        stripe_customer_id,
        user_id
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        Err(sqlx::Error::RowNotFound)
    } else {
        Ok(())
    }
}

/// Deletes a user from the database.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `user_id` - The UUID of the user to delete.
///
/// # Returns
///
/// A `Result` containing the number of rows affected on success, or an `sqlx::Error` on failure.
/// Returns 0 if the user was not found.
#[instrument(skip(pool))]
pub async fn delete_user(pool: &PgPool, user_id: Uuid) -> Result<u64, Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM users 
        WHERE id = $1
        "#,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
