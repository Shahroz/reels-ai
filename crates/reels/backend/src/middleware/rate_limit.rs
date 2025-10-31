use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use actix_web::HttpResponse;
use tracing::{info, warn};

/// Rate limit configuration
pub const RATE_LIMIT_REQUESTS_PER_DAY: u32 = 10;
pub const RATE_LIMIT_CLEANUP_INTERVAL_HOURS: u64 = 24;

/// Represents a rate limit entry for an IP address
#[derive(Debug, Clone)]
struct RateLimitEntry {
    count: u32,
    first_request_time: Option<u64>, // Unix timestamp
    last_request_time: Option<u64>,  // Unix timestamp
}

impl RateLimitEntry {
    fn new() -> Self {
        Self {
            count: 0,
            first_request_time: None,
            last_request_time: None,
        }
    }

    fn increment(&mut self) {
        self.count += 1;
        let now = current_timestamp();
        
        // Set first_request_time only on the first request
        if self.first_request_time.is_none() {
            self.first_request_time = Some(now);
        }
        
        self.last_request_time = Some(now);
    }

    fn is_expired(&self) -> bool {
        if let Some(first_time) = self.first_request_time {
            let now = current_timestamp();
            let day_in_seconds = 24 * 60 * 60;
            now - first_time > day_in_seconds
        } else {
            false // If no first_request_time, it's not expired
        }
    }

    fn is_limit_exceeded(&self) -> bool {
        self.count >= RATE_LIMIT_REQUESTS_PER_DAY
    }
}

/// In-memory rate limiter that tracks requests per IP address per day
#[derive(Debug, Clone)]
pub struct RateLimiter {
    entries: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        let limiter = Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Start cleanup task
        limiter.start_cleanup_task();
        
        limiter
    }

    /// Create a new rate limiter without starting the cleanup task (for testing)
    #[cfg(test)]
    pub fn new_for_testing() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if the IP address has exceeded the rate limit
    /// Returns true if the request should be allowed, false if rate limited
    pub fn check_rate_limit(&self, ip: &str) -> bool {
        let mut entries = self.entries.write().unwrap();
        
        // Clean up expired entries first
        self.cleanup_expired_entries(&mut entries);
        
        let entry = entries.entry(ip.to_string()).or_insert_with(RateLimitEntry::new);
        
        if entry.is_limit_exceeded() {
            warn!(
                "Rate limit exceeded for IP: {}. Count: {}, Limit: {}",
                ip, entry.count, RATE_LIMIT_REQUESTS_PER_DAY
            );
            return false;
        }
        
        entry.increment();
        
        info!(
            "Rate limit check for IP: {}. Count: {}/{}",
            ip, entry.count, RATE_LIMIT_REQUESTS_PER_DAY
        );
        
        true
    }

    /// Get current count for an IP address (for debugging)
    pub fn get_count(&self, ip: &str) -> u32 {
        let entries = self.entries.read().unwrap();
        entries.get(ip).map(|entry| entry.count).unwrap_or(0)
    }

    /// Clear all rate limit entries (for debugging/admin purposes)
    pub fn clear_cache(&self) {
        let mut entries = self.entries.write().unwrap();
        let count = entries.len();
        entries.clear();
        info!("Cleared {} rate limit entries from cache", count);
    }

    /// Clean up expired entries
    fn cleanup_expired_entries(&self, entries: &mut HashMap<String, RateLimitEntry>) {
        let expired_ips: Vec<String> = entries
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(ip, _)| ip.clone())
            .collect();
        
        for ip in expired_ips {
            entries.remove(&ip);
            info!("Cleaned up expired rate limit entry for IP: {}", ip);
        }
    }

    /// Start background cleanup task
    fn start_cleanup_task(&self) {
        let entries = Arc::clone(&self.entries);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(
                RATE_LIMIT_CLEANUP_INTERVAL_HOURS * 60 * 60
            ));
            
            loop {
                interval.tick().await;
                
                let mut entries_guard = entries.write().unwrap();
                
                // Clean up expired entries directly
                let expired_ips: Vec<String> = entries_guard
                    .iter()
                    .filter(|(_, entry)| entry.is_expired())
                    .map(|(ip, _)| ip.clone())
                    .collect();
                
                for ip in expired_ips {
                    entries_guard.remove(&ip);
                    info!("Cleaned up expired rate limit entry for IP: {}", ip);
                }
                
                info!("Performed scheduled rate limit cleanup");
            }
        });
    }
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Create rate limit exceeded response
pub fn rate_limit_exceeded_response() -> HttpResponse {
    HttpResponse::TooManyRequests()
        .json(serde_json::json!({
            "error": "Rate limit exceeded",
            "message": format!("Maximum {} requests per day allowed! Sign up now to get unlimited access to Bounti Studio.", RATE_LIMIT_REQUESTS_PER_DAY),
            "retry_after": "24 hours"
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_entry_creation() {
        let entry = RateLimitEntry::new();
        assert_eq!(entry.count, 0);
        assert!(!entry.is_limit_exceeded());
        assert!(entry.first_request_time.is_none());
        assert!(entry.last_request_time.is_none());
    }

    #[test]
    fn test_rate_limit_entry_increment() {
        let mut entry = RateLimitEntry::new();
        
        // First increment should set timestamps
        entry.increment();
        assert_eq!(entry.count, 1);
        assert!(entry.first_request_time.is_some());
        assert!(entry.last_request_time.is_some());
        assert!(!entry.is_limit_exceeded());
        
        // Increment up to the limit
        for _ in 2..=RATE_LIMIT_REQUESTS_PER_DAY {
            entry.increment();
        }
        assert_eq!(entry.count, RATE_LIMIT_REQUESTS_PER_DAY);
        assert!(entry.is_limit_exceeded());
    }

    #[test]
    fn test_rate_limiter_check() {
        let limiter = RateLimiter::new_for_testing();
        let test_ip = "192.168.1.1";
        
        // Should allow requests up to the limit
        for i in 1..=RATE_LIMIT_REQUESTS_PER_DAY {
            assert!(limiter.check_rate_limit(test_ip), "Request {} should be allowed", i);
        }
        
        // Should reject the next request (after limit)
        assert!(!limiter.check_rate_limit(test_ip), "Request after limit should be rejected");
        
        // Verify count is still at the limit (allowed requests, next rejected without incrementing)
        assert_eq!(limiter.get_count(test_ip), RATE_LIMIT_REQUESTS_PER_DAY);
    }

    #[test]
    fn test_rate_limiter_unknown_ip() {
        let limiter = RateLimiter::new_for_testing();
        
        // Unknown IP should start with count 0 and allow the first request
        assert_eq!(limiter.get_count("unknown"), 0);
        
        // First request should be allowed and set timestamps
        assert!(limiter.check_rate_limit("unknown"));
        assert_eq!(limiter.get_count("unknown"), 1);
    }

    #[test]
    fn test_rate_limit_reset_after_24_hours() {
        let limiter = RateLimiter::new_for_testing();
        let test_ip = "192.168.1.100";
        
        // Exhaust the rate limit
        for i in 1..=RATE_LIMIT_REQUESTS_PER_DAY {
            assert!(limiter.check_rate_limit(test_ip), "Request {} should be allowed", i);
        }
        
        // Verify limit is exceeded (next request rejected)
        assert!(!limiter.check_rate_limit(test_ip), "Request after limit should be rejected");
        assert_eq!(limiter.get_count(test_ip), RATE_LIMIT_REQUESTS_PER_DAY);
        
        // Simulate 24+ hours passing by manually creating an expired entry
        let mut entries = limiter.entries.write().unwrap();
        if let Some(entry) = entries.get_mut(test_ip) {
            // Set first_request_time to 25 hours ago
            let now = current_timestamp();
            let twenty_five_hours_ago = now - (25 * 60 * 60);
            entry.first_request_time = Some(twenty_five_hours_ago);
        }
        drop(entries);
        
        // Check rate limit again - should trigger cleanup and allow the request
        assert!(limiter.check_rate_limit(test_ip), "Request should be allowed after 24+ hours");
        
        // Count should be reset to 1 (the new request)
        assert_eq!(limiter.get_count(test_ip), 1);
    }

    #[test]
    fn test_expired_entry_cleanup() {
        let limiter = RateLimiter::new_for_testing();
        let test_ip = "192.168.1.200";
        
        // Create an entry
        assert!(limiter.check_rate_limit(test_ip));
        assert_eq!(limiter.get_count(test_ip), 1);
        
        // Manually expire the entry
        let mut entries = limiter.entries.write().unwrap();
        if let Some(entry) = entries.get_mut(test_ip) {
            let now = current_timestamp();
            let twenty_five_hours_ago = now - (25 * 60 * 60);
            entry.first_request_time = Some(twenty_five_hours_ago);
        }
        drop(entries);
        
        // Verify entry is expired
        let entries = limiter.entries.read().unwrap();
        if let Some(entry) = entries.get(test_ip) {
            assert!(entry.is_expired(), "Entry should be expired");
        }
        drop(entries);
        
        // Check rate limit - should clean up expired entry and create new one
        assert!(limiter.check_rate_limit(test_ip), "Request should be allowed after cleanup");
        assert_eq!(limiter.get_count(test_ip), 1);
    }

    #[test]
    fn test_clear_cache() {
        let limiter = RateLimiter::new_for_testing();
        let test_ip1 = "192.168.1.300";
        let test_ip2 = "192.168.1.301";
        
        // Create entries for multiple IPs
        assert!(limiter.check_rate_limit(test_ip1));
        assert!(limiter.check_rate_limit(test_ip2));
        
        // Verify entries exist
        assert_eq!(limiter.get_count(test_ip1), 1);
        assert_eq!(limiter.get_count(test_ip2), 1);
        
        // Clear the cache
        limiter.clear_cache();
        
        // Verify all entries are cleared
        assert_eq!(limiter.get_count(test_ip1), 0);
        assert_eq!(limiter.get_count(test_ip2), 0);
        
        // Verify new requests work after cache clear
        assert!(limiter.check_rate_limit(test_ip1));
        assert_eq!(limiter.get_count(test_ip1), 1);
    }
}
