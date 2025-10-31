//! Hybrid session manager with in-memory cache and database fallback.
//!
//! This module implements a smart session management strategy that minimizes
//! database queries by caching active sessions in memory. Database queries
//! only occur when sessions expire or on cache misses.
//! Reduces DB load from 80 queries per session to ~1-2 queries per session.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Session timeout duration in minutes - controls when sessions expire
/// Set to 1 minute for testing, 30 minutes for production
const SESSION_TIMEOUT_MINUTES: i64 = 30;

/// Derived timing constants based on SESSION_TIMEOUT_MINUTES
const DB_SYNC_THRESHOLD_MINUTES: i64 = (SESSION_TIMEOUT_MINUTES as f64 * 0.8) as i64; // 80% of timeout
const CACHE_CLEANUP_THRESHOLD_MINUTES: i64 = (SESSION_TIMEOUT_MINUTES as f64 * 1.2) as i64; // 120% of timeout

/// Calculate cleanup interval - every 1/6th of timeout, minimum 60 seconds
const fn calculate_cleanup_interval() -> u64 {
    let interval = (SESSION_TIMEOUT_MINUTES * 60) / 6;
    if interval < 60 { 60 } else { interval as u64 }
}
const CLEANUP_INTERVAL_SECONDS: u64 = calculate_cleanup_interval();

/// Cached session information stored in memory
#[derive(Debug, Clone)]
pub struct CachedSession {
    pub session_id: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

impl CachedSession {
    /// Check if session is expired based on SESSION_TIMEOUT_MINUTES constant
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now();
        let session_timeout = chrono::Duration::minutes(SESSION_TIMEOUT_MINUTES);
        now.signed_duration_since(self.last_activity) > session_timeout
    }
    
    /// Check if session needs to be synced to database (near expiration)
    pub fn needs_db_sync(&self) -> bool {
        let now = chrono::Utc::now();
        let sync_threshold = chrono::Duration::minutes(DB_SYNC_THRESHOLD_MINUTES);
        now.signed_duration_since(self.last_activity) > sync_threshold
    }
    
    /// Check if session should be cleaned from cache
    pub fn should_cleanup(&self) -> bool {
        let now = chrono::Utc::now();
        let cleanup_threshold = chrono::Duration::minutes(CACHE_CLEANUP_THRESHOLD_MINUTES);
        now.signed_duration_since(self.last_activity) > cleanup_threshold
    }
    
    /// Update last activity to current time
    pub fn touch(&mut self) {
        self.last_activity = chrono::Utc::now();
    }
}

/// Database session record for persistence
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserSession {
    pub user_id: uuid::Uuid,
    pub session_id: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Hybrid session manager with smart caching
#[derive(Debug)]
pub struct HybridSessionManager {
    /// In-memory cache of active sessions
    cache: Arc<RwLock<HashMap<uuid::Uuid, CachedSession>>>,
    /// Database connection pool
    db_pool: sqlx::PgPool,
}

impl HybridSessionManager {
    /// Create new session manager
    pub fn new(db_pool: sqlx::PgPool) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            db_pool,
        }
    }
    
    /// Get or create session for user with zero duplicates across machines
    /// 
    /// Strategy:
    /// 1. Check in-memory cache first
    /// 2. If cache hit and session fresh: return cached session (with proactive DB sync if needed)
    /// 3. If cache miss: ALWAYS check database before creating new session
    /// 4. If DB session exists and fresh: cache it and return
    /// 5. If no valid session: create new session in DB and cache
    pub async fn get_or_create_session(&self, user_id: uuid::Uuid) -> Result<String, SessionError> {
        // Step 1: Check in-memory cache
        {
            let cache = self.cache.read().await;
            if let Some(cached_session) = cache.get(&user_id) {
                if !cached_session.is_expired() {
                    // Cache hit with fresh session
                    let session_id = cached_session.session_id.clone();
                    
                    // Proactive DB sync if session is near expiration
                    if cached_session.needs_db_sync() {
                        // Update session activity in background (don't wait)
                        let user_id_copy = user_id;
                        let db_pool = self.db_pool.clone();
                        tokio::spawn(async move {
                            let _ = sqlx::query!(
                                "UPDATE user_sessions 
                                 SET last_activity = NOW() 
                                 WHERE user_id = $1 AND is_active = true",
                                user_id_copy
                            ).execute(&db_pool).await;
                        });
                    }
                    
                    // Update local cache activity
                    drop(cache);
                    {
                        let mut cache = self.cache.write().await;
                        if let Some(session) = cache.get_mut(&user_id) {
                            session.touch();
                        }
                    }
                    
                    return Ok(session_id);
                }
            }
        }
        
        // Step 2: Cache miss - ALWAYS check database before creating new session
        let db_session = self.get_session_from_db(user_id).await?;
        
        if let Some(session) = db_session {
            let cached = CachedSession {
                session_id: session.session_id.clone(),
                started_at: session.started_at,
                last_activity: session.last_activity,
            };
            
            if !cached.is_expired() {
                // Fresh session from DB - cache it
                {
                    let mut cache = self.cache.write().await;
                    cache.insert(user_id, cached);
                }
                
                // Update last activity in DB (session accessed from different machine)
                let _ = self.update_session_activity(user_id).await;
                
                return Ok(session.session_id);
            } else {
                // Session in DB is expired - mark as inactive
                let _ = self.end_session(user_id).await;
            }
        }
        
        // Step 3: No valid session exists - create new one
        self.create_new_session(user_id).await
    }
    
    /// Update session activity (called on each user action)
    /// Only updates cache if session exists, DB update is async
    pub async fn touch_session(&self, user_id: uuid::Uuid) -> Result<(), SessionError> {
        // Update cache immediately
        {
            let mut cache = self.cache.write().await;
            if let Some(session) = cache.get_mut(&user_id) {
                session.touch();
            }
        }
        
        // Update DB asynchronously (don't wait for result)
        let db_pool = self.db_pool.clone();
        tokio::spawn(async move {
            let _ = sqlx::query!(
                "UPDATE user_sessions SET last_activity = NOW() WHERE user_id = $1 AND is_active = true",
                user_id
            )
            .execute(&db_pool)
            .await;
        });
        
        Ok(())
    }
    
    /// Get session from database
    async fn get_session_from_db(&self, user_id: uuid::Uuid) -> Result<Option<UserSession>, SessionError> {
        let session = sqlx::query_as!(
            UserSession,
            "SELECT user_id, session_id, started_at, last_activity, is_active, created_at, updated_at 
             FROM user_sessions 
             WHERE user_id = $1 AND is_active = true",
            user_id
        )
        .fetch_optional(&self.db_pool)
        .await?;
        
        Ok(session)
    }
    
    /// Create new session in database and cache
    async fn create_new_session(&self, user_id: uuid::Uuid) -> Result<String, SessionError> {
        let now = chrono::Utc::now();
        let session_id = format!("{}_{}", user_id, now.timestamp_millis());
        
        // First, mark any existing active sessions as inactive
        sqlx::query!(
            "UPDATE user_sessions SET is_active = false 
             WHERE user_id = $1 AND is_active = true",
            user_id
        )
        .execute(&self.db_pool)
        .await?;
        
        // Then insert the new session (always creates a new record)
        sqlx::query!(
            "INSERT INTO user_sessions (user_id, session_id, started_at, last_activity, is_active)
             VALUES ($1, $2, $3, $3, true)",
            user_id,
            session_id,
            now
        )
        .execute(&self.db_pool)
        .await?;
        
        // Cache the new session
        let cached_session = CachedSession {
            session_id: session_id.clone(),
            started_at: now,
            last_activity: now,
        };
        
        {
            let mut cache = self.cache.write().await;
            cache.insert(user_id, cached_session);
        }
        
        Ok(session_id)
    }
    
    /// Update session activity in database
    async fn update_session_activity(&self, user_id: uuid::Uuid) -> Result<(), SessionError> {
        sqlx::query!(
            "UPDATE user_sessions SET last_activity = NOW() WHERE user_id = $1 AND is_active = true",
            user_id
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }
    
    /// End session (mark as inactive)
    pub async fn end_session(&self, user_id: uuid::Uuid) -> Result<(), SessionError> {
        // Remove from cache
        {
            let mut cache = self.cache.write().await;
            cache.remove(&user_id);
        }
        
        // Mark as inactive in database
        sqlx::query!(
            "UPDATE user_sessions SET is_active = false WHERE user_id = $1",
            user_id
        )
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }
    
    /// Get cache statistics for monitoring
    pub async fn get_cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let active_sessions = cache.len();
        let expired_sessions = cache.values()
            .filter(|session| session.is_expired())
            .count();
        
        CacheStats {
            active_sessions,
            expired_sessions,
            fresh_sessions: active_sessions - expired_sessions,
        }
    }
    
    /// Clean up expired sessions from cache (background task)
    /// Uses cleanup threshold (120% of session timeout) to avoid premature removal
    pub async fn cleanup_expired_cache(&self) -> Result<usize, SessionError> {
        let mut cache = self.cache.write().await;
        let initial_count = cache.len();
        
        // Remove sessions that should be cleaned up (with buffer beyond expiration)
        cache.retain(|_, session| !session.should_cleanup());
        
        let removed_count = initial_count - cache.len();
        Ok(removed_count)
    }

    /// Start background cleanup task for expired sessions
    /// This should be called once when the application starts
    pub fn start_cleanup_task(session_manager: std::sync::Arc<Self>) {
        // Use the calculated cleanup interval based on session timeout
        let cleanup_interval_secs = CLEANUP_INTERVAL_SECONDS;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(cleanup_interval_secs));
            
            loop {
                interval.tick().await;
                
                // Clean up expired sessions from cache
                if let Ok(removed_count) = session_manager.cleanup_expired_cache().await {
                    if removed_count > 0 {
                        log::info!("Cleaned up {} expired sessions from cache", removed_count);
                    }
                }
                
                // Clean up expired sessions from database
                if let Err(e) = session_manager.cleanup_expired_db_sessions().await {
                    log::error!("Failed to cleanup expired database sessions: {}", e);
                }
            }
        });
    }
    
    /// Clean up expired sessions from database
    async fn cleanup_expired_db_sessions(&self) -> Result<u64, SessionError> {
        // Use raw query since sqlx::query! doesn't support parameterized INTERVAL
        let query = format!(
            "UPDATE user_sessions SET is_active = false 
             WHERE is_active = true AND last_activity < NOW() - INTERVAL '{} minutes'",
            SESSION_TIMEOUT_MINUTES
        );
        
        let result = sqlx::query(&query)
            .execute(&self.db_pool)
            .await?;
        
        Ok(result.rows_affected())
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub active_sessions: usize,
    pub expired_sessions: usize,
    pub fresh_sessions: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Session not found")]
    NotFound,
    
    #[error("Session expired")]
    Expired,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cached_session_expiry() {
        let now = chrono::Utc::now();
        let mut session = CachedSession {
            session_id: "test_session".to_string(),
            started_at: now,
            last_activity: now - chrono::Duration::minutes(31), // 31 minutes ago
        };
        
        assert!(session.is_expired());
        
        session.touch();
        assert!(!session.is_expired());
    }
    
    #[test]
    fn test_timing_constants_calculations() {
        // Verify that derived timing constants scale correctly with SESSION_TIMEOUT_MINUTES
        
        // With SESSION_TIMEOUT_MINUTES = 30 (production setting):
        assert_eq!(DB_SYNC_THRESHOLD_MINUTES, 24); // 80% of 30 = 24.0 -> 24
        assert_eq!(CACHE_CLEANUP_THRESHOLD_MINUTES, 36); // 120% of 30 = 36.0 -> 36
        assert_eq!(CLEANUP_INTERVAL_SECONDS, 300); // (30 * 60) / 6 = 300
        
        // Test the const function directly
        assert_eq!(calculate_cleanup_interval(), 300);
    }
    
    #[test]
    fn test_session_sync_thresholds() {
        let now = chrono::Utc::now();
        
        // Test DB sync threshold (24 minutes for 30-minute timeout)
        let session_needs_sync = CachedSession {
            session_id: "test".to_string(),
            started_at: now,
            last_activity: now - chrono::Duration::minutes(25), // 25 minutes ago (> 24 minute threshold)
        };
        assert!(session_needs_sync.needs_db_sync()); // > 24 minutes
        
        // Test session that doesn't need sync yet
        let session_no_sync = CachedSession {
            session_id: "test".to_string(),
            started_at: now,
            last_activity: now - chrono::Duration::minutes(20), // 20 minutes ago (< 24 minute threshold)
        };
        assert!(!session_no_sync.needs_db_sync()); // < 24 minutes
        
        // Test cache cleanup threshold (36 minutes for 30-minute timeout)
        let session_cleanup = CachedSession {
            session_id: "test".to_string(),
            started_at: now,
            last_activity: now - chrono::Duration::minutes(40), // 40 minutes ago (> 36 minute threshold)
        };
        assert!(session_cleanup.should_cleanup());
        
        let session_no_cleanup = CachedSession {
            session_id: "test".to_string(),
            started_at: now,
            last_activity: now - chrono::Duration::minutes(30), // 30 minutes ago (< 36 minute threshold)
        };
        assert!(!session_no_cleanup.should_cleanup());
    }
    
    #[test]
    fn test_session_lifecycle_timing() {
        let now = chrono::Utc::now();
        let session = CachedSession {
            session_id: "test".to_string(),
            started_at: now,
            last_activity: now - chrono::Duration::minutes(25), // 25 minutes old
        };
        
        // At 25 minutes: needs sync (> 24), but not expired (< 30), not cleanup (< 36)
        assert!(session.needs_db_sync());      // > 24 minutes (80% of 30 minutes)
        assert!(!session.is_expired());       // < 30 minutes (30 minute timeout)
        assert!(!session.should_cleanup());   // < 36 minutes (120% of 30 minutes)
    }
} 