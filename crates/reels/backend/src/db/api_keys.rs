// backend/src/db/api_keys.rs
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{DateTime, Utc};
use rand::distributions::{Alphanumeric, DistString};
use serde::{Serialize, Deserialize};
use sqlx::{postgres::PgPool, types::Uuid, Error, FromRow};
use utoipa::ToSchema; // Added for OpenAPI
use tracing::instrument;

use crate::db::users::{User, find_user_by_id};

// Consistent UserId type based on db/users.rs
pub type UserId = Uuid;

const API_KEY_LENGTH: usize = 64; // Length of the raw API key

/// Represents metadata about an API key (excluding the hash).
#[derive(Debug, FromRow, Serialize, Deserialize, ToSchema)] // Added Deserialize
pub struct ApiKeyMetadata {
    #[schema(value_type = String)] // Correct Uuid representation for OpenAPI
    pub id: Uuid,
    #[schema(value_type = String)] // Correct Uuid representation for OpenAPI
    pub user_id: Uuid,
    #[schema(value_type = String, format = DateTime)] // Explicitly define type for OpenAPI
    pub created_at: DateTime<Utc>,
    #[schema(value_type = Option<String>, format = DateTime)] // Explicitly define type for OpenAPI
    pub last_used_at: Option<DateTime<Utc>>,
    /// Comma-separated list of allowed domains for API key usage
    pub allowed_domains: Option<String>,
    // Add a 'name' or 'description' field here in the future if needed
}

/// Generates a new API key for a user, stores its hash, and returns the raw key.
/// The raw key is only returned *once* upon creation.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `user_id` - The UUID of the user to create the key for.
/// * `allowed_domains` - Optional comma-separated list of allowed domains.
///
/// # Returns
///
/// A `Result` containing the raw, unhashed API key string on success,
/// or an `sqlx::Error` or `bcrypt::BcryptError` wrapped in `sqlx::Error::Protocol` on failure.
#[instrument(skip(pool))]
pub async fn create_api_key(pool: &PgPool, user_id: UserId, allowed_domains: Option<String>) -> Result<String, Error> {
    // 1. Generate a secure random API key string
    let raw_key = Alphanumeric.sample_string(&mut rand::thread_rng(), API_KEY_LENGTH);

    // 2. Hash the generated key using bcrypt
    let key_hash = hash(&raw_key, DEFAULT_COST)
        .map_err(|e| sqlx::Error::Protocol(format!("Bcrypt hash error: {e}")))?;

    // 3. Store the hash, user_id, and allowed_domains in the database
    let result = sqlx::query!(
        r#"
        INSERT INTO api_keys (user_id, key_hash, allowed_domains)
        VALUES ($1::uuid, $2, $3)
        RETURNING id
        "#,
        user_id,
        key_hash,
        allowed_domains
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(_) => Ok(raw_key), // Return the raw key on successful insertion
        Err(e) => {
            eprintln!("Failed to create API key entry: {e}");
            Err(e)
        }
    }
}

/// Validates a provided API key against stored hashes and updates last used time.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `key` - The raw API key string provided by the client.
///
/// # Returns
///
/// A `Result` containing:
/// - `Ok(Some(UserId))` if the key is valid, returning the associated user's ID.
/// - `Ok(None)` if the key is invalid (no matching hash found or bcrypt verification fails).
/// - `Err(sqlx::Error)` on database query errors or bcrypt errors.
#[instrument(skip(pool, key))]
pub async fn validate_api_key(pool: &PgPool, key: &str) -> Result<Option<UserId>, Error> {
    // Note: Inefficiently fetch all key hashes (replace in production!)
    let potential_keys = sqlx::query!(
        r#"
        SELECT user_id AS "user_id: uuid::Uuid", key_hash
        FROM api_keys
        "#
    )
    .fetch_all(pool)
    .await?;

    for record in potential_keys {
        if verify(key, &record.key_hash).unwrap_or(false) {
            // Key is valid, update last_used_at (best effort)
            let _ = sqlx::query!(
                r#"
                UPDATE api_keys
                SET last_used_at = NOW()
                WHERE user_id = $1::uuid AND key_hash = $2
                "#,
                record.user_id,
                record.key_hash
            )
            .execute(pool)
            .await;
            return Ok(Some(record.user_id));
        }
    }

    Ok(None) // No valid key found
}

/// Normalizes a domain by removing protocol, port, and converting to lowercase.
///
/// # Arguments
///
/// * `domain` - The domain string to normalize.
///
/// # Returns
///
/// The normalized domain string.
fn normalize_domain(domain: &str) -> String {
    let mut normalized = domain.to_lowercase();
    
    // Remove protocol (http://, https://, etc.)
    if let Some(protocol_end) = normalized.find("://") {
        normalized = normalized[protocol_end + 3..].to_string();
    }
    
    // Remove port number
    if let Some(port_start) = normalized.rfind(':') {
        // Check if this is likely a port (contains only digits after colon)
        let after_colon = &normalized[port_start + 1..];
        if after_colon.chars().all(|c| c.is_ascii_digit()) {
            // Remove port if it's at the end or followed by /
            if port_start + 1 + after_colon.len() == normalized.len() || after_colon.starts_with('/') {
                normalized = normalized[..port_start].to_string();
            }
        }
    }
    
    // Remove trailing slash
    normalized = normalized.trim_end_matches('/').to_string();
    
    // Remove leading dot
    normalized = normalized.trim_start_matches('.').to_string();
    
    normalized
}

/// Checks if an origin domain matches any of the allowed domains.
///
/// # Arguments
///
/// * `origin` - The origin domain from the request.
/// * `allowed_domains` - Comma-separated list of allowed domains.
///
/// # Returns
///
/// `true` if the origin matches any allowed domain, `false` otherwise.
fn is_domain_allowed(origin: &str, allowed_domains: &str) -> bool {
    let normalized_origin = normalize_domain(origin);
    
    // Split allowed domains and normalize each one
    let allowed_list: Vec<String> = allowed_domains
        .split(',')
        .map(|d| normalize_domain(d.trim()))
        .filter(|d| !d.is_empty())
        .collect();
    
    if allowed_list.is_empty() {
        return false;
    }
    
    // Check for exact match or subdomain match
    allowed_list.iter().any(|allowed_domain| {
        // Exact match
        if normalized_origin == *allowed_domain {
            return true;
        }
        
        // Subdomain match: origin ends with .allowed_domain
        if normalized_origin.ends_with(&format!(".{}", allowed_domain)) {
            return true;
        }
        
        // Wildcard subdomain match: allowed_domain starts with *.
        if allowed_domain.starts_with("*.") {
            let base_domain = &allowed_domain[2..]; // Remove "*."
            if normalized_origin == base_domain || normalized_origin.ends_with(&format!(".{}", base_domain)) {
                return true;
            }
        }
        
        false
    })
}

/// Validates a provided API key against stored hashes and checks domain restrictions.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `key` - The raw API key string provided by the client.
/// * `origin` - The origin domain from the request headers (optional).
///
/// # Returns
///
/// A `Result` containing:
/// - `Ok(Some(User))` if the key is valid and domain is allowed.
/// - `Ok(None)` if the key is invalid or domain is not allowed.
/// - `Err(sqlx::Error)` on database query errors or bcrypt errors.
#[instrument(skip(pool, key))]
pub async fn validate_api_key_with_domain(pool: &PgPool, key: &str, origin: Option<&str>) -> Result<Option<User>, Error> {
    // Note: Inefficiently fetch all key hashes (replace in production!)
    let potential_keys = sqlx::query!(
        r#"
        SELECT user_id AS "user_id: uuid::Uuid", key_hash, allowed_domains
        FROM api_keys
        "#
    )
    .fetch_all(pool)
    .await?;

    for record in potential_keys {
        if verify(key, &record.key_hash).unwrap_or(false) {
            // Key is valid, check domain restrictions
            if let Some(allowed_domains) = &record.allowed_domains {
                if let Some(origin_domain) = origin {
                    // Check if the origin domain is in the allowed list
                    if !is_domain_allowed(origin_domain, allowed_domains) {
                        log::warn!(
                            "API key domain restriction: origin '{}' not in allowed domains '{}'", 
                            origin_domain, 
                            allowed_domains
                        );
                        return Ok(None);
                    }
                } else {
                    // No origin provided but domain restrictions exist
                    log::warn!("API key has domain restrictions but no origin provided");
                    return Ok(None);
                }
            }
            
            // Key is valid and domain is allowed (or no restrictions), update last_used_at
            let _ = sqlx::query!(
                r#"
                UPDATE api_keys
                SET last_used_at = NOW()
                WHERE user_id = $1::uuid AND key_hash = $2
                "#,
                record.user_id,
                record.key_hash
            )
            .execute(pool)
            .await;
            
            // Fetch user details after successful validation
            match find_user_by_id(pool, record.user_id).await {
                Ok(Some(user)) => return Ok(Some(user)),
                Ok(None) => {
                    log::error!("User not found after API key validation: {}", record.user_id);
                    return Ok(None);
                }
                Err(e) => {
                    log::error!("Failed to fetch user details after API key validation: {}", e);
                    return Err(e);
                }
            }
        }
    }

    Ok(None) // No valid key found
}

/// Lists API key metadata for a specific user.
/// Does not return the key hash or the raw key.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `user_id` - The UUID of the user whose keys to list.
///
/// # Returns
///
/// A `Result` containing a vector of `ApiKeyMetadata` on success, or `sqlx::Error`.
#[instrument(skip(pool))]
pub async fn list_api_keys_for_user(
    pool: &PgPool,
    user_id: UserId,
) -> Result<Vec<ApiKeyMetadata>, Error> {
    let keys = sqlx::query_as!(
        ApiKeyMetadata,
        r#"
        SELECT id AS "id: uuid::Uuid", user_id AS "user_id: uuid::Uuid", created_at, last_used_at, allowed_domains
        FROM api_keys
        WHERE user_id = $1::uuid
        ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(keys)
}

/// Deletes a specific API key belonging to a user.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `user_id` - The UUID of the user owning the key.
/// * `key_id` - The UUID of the API key to delete.
///
/// # Returns
///
/// A `Result` containing:
/// - `Ok(true)` if the key was found and deleted.
/// - `Ok(false)` if no key matching the `user_id` and `key_id` was found.
/// - `Err(sqlx::Error)` on database errors.
#[instrument(skip(pool))]
pub async fn delete_api_key(pool: &PgPool, user_id: UserId, key_id: Uuid) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM api_keys
        WHERE id = $1::uuid AND user_id = $2::uuid
        "#,
        key_id,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Lists all API keys in the system (admin only).
///
/// # Arguments
///
/// * `pool` - The database connection pool.
///
/// # Returns
///
/// A `Result` containing a vector of `ApiKeyMetadata` on success, or `sqlx::Error`.
#[instrument(skip(pool))]
pub async fn list_all_api_keys(pool: &PgPool) -> Result<Vec<ApiKeyMetadata>, Error> {
    let keys = sqlx::query_as!(
        ApiKeyMetadata,
        r#"
        SELECT id AS "id: uuid::Uuid", user_id AS "user_id: uuid::Uuid", created_at, last_used_at, allowed_domains
        FROM api_keys
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(keys)
}

/// Deletes any API key by ID (admin only).
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `key_id` - The UUID of the API key to delete.
///
/// # Returns
///
/// A `Result` containing:
/// - `Ok(true)` if the key was found and deleted.
/// - `Ok(false)` if no key matching the `key_id` was found.
/// - `Err(sqlx::Error)` on database errors.
#[instrument(skip(pool))]
pub async fn delete_any_api_key(pool: &PgPool, key_id: Uuid) -> Result<bool, Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM api_keys
        WHERE id = $1::uuid
        "#,
        key_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_domain() {
        // Test protocol removal
        assert_eq!(normalize_domain("https://example.com"), "example.com");
        assert_eq!(normalize_domain("http://example.com"), "example.com");
        assert_eq!(normalize_domain("ftp://example.com"), "example.com");
        
        // Test port removal
        assert_eq!(normalize_domain("example.com:8080"), "example.com");
        assert_eq!(normalize_domain("https://example.com:443"), "example.com");
        
        // Test trailing slash removal
        assert_eq!(normalize_domain("example.com/"), "example.com");
        assert_eq!(normalize_domain("https://example.com/path/"), "example.com/path");
        
        // Test leading dot removal
        assert_eq!(normalize_domain(".example.com"), "example.com");
        
        // Test case conversion
        assert_eq!(normalize_domain("EXAMPLE.COM"), "example.com");
        assert_eq!(normalize_domain("Example.Com"), "example.com");
        
        // Test complex case
        assert_eq!(normalize_domain("HTTPS://EXAMPLE.COM:8080/PATH/"), "example.com:8080/path");
    }

    #[test]
    fn test_is_domain_allowed() {
        // Test exact match
        assert!(is_domain_allowed("example.com", "example.com"));
        assert!(is_domain_allowed("EXAMPLE.COM", "example.com"));
        assert!(is_domain_allowed("https://example.com", "example.com"));
        assert!(is_domain_allowed("https://example.com:443", "example.com"));
        
        // Test subdomain match
        assert!(is_domain_allowed("api.example.com", "example.com"));
        assert!(is_domain_allowed("sub.api.example.com", "example.com"));
        assert!(is_domain_allowed("https://api.example.com", "example.com"));
        
        // Test wildcard subdomain
        assert!(is_domain_allowed("api.example.com", "*.example.com"));
        assert!(is_domain_allowed("sub.api.example.com", "*.example.com"));
        assert!(is_domain_allowed("example.com", "*.example.com"));
        
        // Test multiple allowed domains
        assert!(is_domain_allowed("example.com", "example.com,test.com"));
        assert!(is_domain_allowed("test.com", "example.com,test.com"));
        assert!(is_domain_allowed("api.example.com", "example.com,test.com"));
        
        // Test negative cases
        assert!(!is_domain_allowed("other.com", "example.com"));
        assert!(!is_domain_allowed("example.org", "example.com"));
        assert!(!is_domain_allowed("malicious.example.com.evil.com", "example.com"));
        
        // Test empty or invalid domains
        assert!(!is_domain_allowed("example.com", ""));
        assert!(!is_domain_allowed("example.com", ","));
        assert!(!is_domain_allowed("", "example.com"));
    }
}

/// Lists API keys for a specific user with user details.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `user_id` - The UUID of the user to list keys for.
///
/// # Returns
///
/// A `Result` containing a vector of `ApiKeyWithUserDetails` on success,
/// or an `sqlx::Error` on failure.
#[instrument(skip(pool))]
pub async fn list_api_keys_with_user_details_for_user(
    pool: &PgPool,
    user_id: UserId,
) -> Result<Vec<crate::routes::api_keys::list_keys_response::ApiKeyWithUserDetails>, Error> {
    let keys = sqlx::query_as!(
        crate::routes::api_keys::list_keys_response::ApiKeyWithUserDetails,
        r#"
        SELECT 
            ak.id AS "id: uuid::Uuid",
            ak.user_id AS "user_id: uuid::Uuid",
            ak.created_at,
            ak.last_used_at,
            ak.allowed_domains,
            u.email AS "user_email",
            u.email_verified AS "user_email_verified",
            u.is_admin AS "user_is_admin",
            u.status AS "user_status",
            u.created_at AS "user_created_at"
        FROM api_keys ak
        JOIN users u ON ak.user_id = u.id
        WHERE ak.user_id = $1::uuid
        ORDER BY ak.created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(keys)
}

/// Lists all API keys with user details (admin only).
///
/// # Arguments
///
/// * `pool` - The database connection pool.
///
/// # Returns
///
/// A `Result` containing a vector of `ApiKeyWithUserDetails` on success,
/// or an `sqlx::Error` on failure.
#[instrument(skip(pool))]
pub async fn list_all_api_keys_with_user_details(
    pool: &PgPool,
) -> Result<Vec<crate::routes::api_keys::list_keys_response::ApiKeyWithUserDetails>, Error> {
    let keys = sqlx::query_as!(
        crate::routes::api_keys::list_keys_response::ApiKeyWithUserDetails,
        r#"
        SELECT 
            ak.id AS "id: uuid::Uuid",
            ak.user_id AS "user_id: uuid::Uuid",
            ak.created_at,
            ak.last_used_at,
            ak.allowed_domains,
            u.email AS "user_email",
            u.email_verified AS "user_email_verified",
            u.is_admin AS "user_is_admin",
            u.status AS "user_status",
            u.created_at AS "user_created_at"
        FROM api_keys ak
        JOIN users u ON ak.user_id = u.id
        ORDER BY ak.created_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(keys)
}

/// Lists API keys with user details for a specific user with optional email search.
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `user_id` - The UUID of the user to list keys for.
/// * `search` - Optional search term to filter by user email (case insensitive).
///
/// # Returns
///
/// A `Result` containing a vector of `ApiKeyWithUserDetails` on success,
/// or an `sqlx::Error` on failure.
#[instrument(skip(pool))]
pub async fn list_api_keys_with_user_details_for_user_search(
    pool: &PgPool,
    user_id: UserId,
    search: Option<&str>,
) -> Result<Vec<crate::routes::api_keys::list_keys_response::ApiKeyWithUserDetails>, Error> {
    let keys = if let Some(search_term) = search {
        // Search case-insensitively in user email
        sqlx::query_as!(
            crate::routes::api_keys::list_keys_response::ApiKeyWithUserDetails,
            r#"
            SELECT 
                ak.id AS "id: uuid::Uuid",
                ak.user_id AS "user_id: uuid::Uuid",
                ak.created_at,
                ak.last_used_at,
                ak.allowed_domains,
                u.email AS "user_email",
                u.email_verified AS "user_email_verified",
                u.is_admin AS "user_is_admin",
                u.status AS "user_status",
                u.created_at AS "user_created_at"
            FROM api_keys ak
            JOIN users u ON ak.user_id = u.id
            WHERE ak.user_id = $1::uuid
            AND LOWER(u.email) LIKE LOWER($2)
            ORDER BY ak.created_at DESC
            "#,
            user_id,
            format!("%{}%", search_term)
        )
        .fetch_all(pool)
        .await?
    } else {
        // No search term, return all keys for user
        sqlx::query_as!(
            crate::routes::api_keys::list_keys_response::ApiKeyWithUserDetails,
            r#"
            SELECT 
                ak.id AS "id: uuid::Uuid",
                ak.user_id AS "user_id: uuid::Uuid",
                ak.created_at,
                ak.last_used_at,
                ak.allowed_domains,
                u.email AS "user_email",
                u.email_verified AS "user_email_verified",
                u.is_admin AS "user_is_admin",
                u.status AS "user_status",
                u.created_at AS "user_created_at"
            FROM api_keys ak
            JOIN users u ON ak.user_id = u.id
            WHERE ak.user_id = $1::uuid
            ORDER BY ak.created_at DESC
            "#,
            user_id
        )
        .fetch_all(pool)
        .await?
    };

    Ok(keys)
}

/// Lists all API keys with user details with optional email search (admin only).
///
/// # Arguments
///
/// * `pool` - The database connection pool.
/// * `search` - Optional search term to filter by user email (case insensitive).
///
/// # Returns
///
/// A `Result` containing a vector of `ApiKeyWithUserDetails` on success,
/// or an `sqlx::Error` on failure.
#[instrument(skip(pool))]
pub async fn list_all_api_keys_with_user_details_search(
    pool: &PgPool,
    search: Option<&str>,
) -> Result<Vec<crate::routes::api_keys::list_keys_response::ApiKeyWithUserDetails>, Error> {
    let keys = if let Some(search_term) = search {
        // Search case-insensitively in user email
        sqlx::query_as!(
            crate::routes::api_keys::list_keys_response::ApiKeyWithUserDetails,
            r#"
            SELECT 
                ak.id AS "id: uuid::Uuid",
                ak.user_id AS "user_id: uuid::Uuid",
                ak.created_at,
                ak.last_used_at,
                ak.allowed_domains,
                u.email AS "user_email",
                u.email_verified AS "user_email_verified",
                u.is_admin AS "user_is_admin",
                u.status AS "user_status",
                u.created_at AS "user_created_at"
            FROM api_keys ak
            JOIN users u ON ak.user_id = u.id
            WHERE LOWER(u.email) LIKE LOWER($1)
            ORDER BY ak.created_at DESC
            "#,
            format!("%{}%", search_term)
        )
        .fetch_all(pool)
        .await?
    } else {
        // No search term, return all keys
        sqlx::query_as!(
            crate::routes::api_keys::list_keys_response::ApiKeyWithUserDetails,
            r#"
            SELECT 
                ak.id AS "id: uuid::Uuid",
                ak.user_id AS "user_id: uuid::Uuid",
                ak.created_at,
                ak.last_used_at,
                ak.allowed_domains,
                u.email AS "user_email",
                u.email_verified AS "user_email_verified",
                u.is_admin AS "user_is_admin",
                u.status AS "user_status",
                u.created_at AS "user_created_at"
            FROM api_keys ak
            JOIN users u ON ak.user_id = u.id
            ORDER BY ak.created_at DESC
            "#
        )
        .fetch_all(pool)
        .await?
    };

    Ok(keys)
}
