//! Defines queries for cohort registration analysis
//! 
//! Provides functionality to analyze user registration events based on user cohorts
//! defined by registration date ranges, with breakdown by device, OS, browser, and country.
//!
//! Revision History
//! - 2025-09-22T00:00:00Z @AI: Initial implementation of cohort registration analysis.


/// Represents cohort registration analysis statistics with dimension breakdowns
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CohortRegistrationStat {
    pub dimension_name: String,
    pub dimension_value: String,
    pub total_events: i64,
    pub unique_users: i64,
}

/// IP to country lookup service trait for external geolocation
pub trait IpToCountryService: Send + Sync {
    async fn get_country_from_ip(&self, ip: &str) -> Option<String>;
}

/// Simple mock implementation for IP to country lookup
pub struct MockIpToCountryService;

impl IpToCountryService for MockIpToCountryService {
    async fn get_country_from_ip(&self, ip: &str) -> Option<String> {
        // This is a mock implementation - in production you'd want to use
        // a real geolocation service like MaxMind GeoIP2, ip-api.com, etc.
        match ip {
            ip if ip.starts_with("209.") => Some("United States".to_string()),
            ip if ip.starts_with("50.") => Some("United States".to_string()),
            ip if ip.starts_with("72.") => Some("United States".to_string()),
            ip if ip.starts_with("192.168.") => Some("Local Network".to_string()),
            ip if ip.starts_with("10.") => Some("Local Network".to_string()),
            _ => Some("Unknown".to_string()),
        }
    }
}

// db_ip implementation (no signup required - embedded database)
use db_ip::{include_country_code_database, CountryCode};

pub struct DbIpService {
    db: db_ip::DbIpDatabase<CountryCode>,
}

impl DbIpService {
    pub fn new() -> Self {
        // Database embedded in binary - no external files or signups needed!
        let db = include_country_code_database!();
        Self { db }
    }
}

impl IpToCountryService for DbIpService {
    async fn get_country_from_ip(&self, ip: &str) -> Option<String> {
        if let Ok(ip_addr) = ip.parse() {
            if let Some(country_code) = self.db.get(&ip_addr) {
                // Convert country code (e.g., "US") to country name (e.g., "United States")
                let code_str = country_code.to_string();
                let country_name = match code_str.as_str() {
                    "US" => "United States",
                    "CA" => "Canada",
                    "GB" => "United Kingdom",
                    "DE" => "Germany",
                    "FR" => "France",
                    "JP" => "Japan",
                    "AU" => "Australia",
                    "BR" => "Brazil",
                    "IN" => "India",
                    "CN" => "China",
                    "RU" => "Russia",
                    "IT" => "Italy",
                    "ES" => "Spain",
                    "MX" => "Mexico",
                    "NL" => "Netherlands",
                    "SE" => "Sweden",
                    "NO" => "Norway",
                    "DK" => "Denmark",
                    "FI" => "Finland",
                    "CH" => "Switzerland",
                    "AT" => "Austria",
                    "BE" => "Belgium",
                    "IE" => "Ireland",
                    "PL" => "Poland",
                    "PT" => "Portugal",
                    "CZ" => "Czech Republic",
                    "HU" => "Hungary",
                    "GR" => "Greece",
                    "TR" => "Turkey",
                    "IL" => "Israel",
                    "ZA" => "South Africa",
                    "EG" => "Egypt",
                    "NG" => "Nigeria",
                    "KE" => "Kenya",
                    "AR" => "Argentina",
                    "CL" => "Chile",
                    "CO" => "Colombia",
                    "PE" => "Peru",
                    "VE" => "Venezuela",
                    "TH" => "Thailand",
                    "VN" => "Vietnam",
                    "SG" => "Singapore",
                    "MY" => "Malaysia",
                    "ID" => "Indonesia",
                    "PH" => "Philippines",
                    "KR" => "South Korea",
                    "TW" => "Taiwan",
                    "HK" => "Hong Kong",
                    "NZ" => "New Zealand",
                    "UA" => "Ukraine",
                    "RO" => "Romania",
                    "BG" => "Bulgaria",
                    "HR" => "Croatia",
                    "SI" => "Slovenia",
                    "SK" => "Slovakia",
                    "EE" => "Estonia",
                    "LV" => "Latvia",
                    "LT" => "Lithuania",
                    code => code, // Fallback to country code if name not mapped
                };
                return Some(country_name.to_string());
            }
        }
        Some("Unknown".to_string())
    }
}

// Example ipgeolocate implementation (API-based, no signup required)
/*
use ipgeolocate::{Locator, Service};

pub struct IpGeolocateService;

impl IpToCountryService for IpGeolocateService {
    async fn get_country_from_ip(&self, ip: &str) -> Option<String> {
        match Locator::get(ip, Service::IpApi).await {
            Ok(info) => Some(info.country),
            Err(_) => Some("Unknown".to_string()),
        }
    }
}
*/


/// Fetches cohort registration analysis by device type
#[tracing::instrument(
    name = "query_cohort_registration_by_device",
    skip(pool),
    fields(
        start_cohort_date = %start_cohort_date,
        end_cohort_date = %end_cohort_date,
        start_event_date = %start_event_date,
        end_event_date = %end_event_date
    )
)]
pub async fn query_cohort_registration_by_device(
    pool: &sqlx::PgPool,
    start_cohort_date: chrono::DateTime<chrono::Utc>,
    end_cohort_date: chrono::DateTime<chrono::Utc>,
    start_event_date: chrono::DateTime<chrono::Utc>,
    end_event_date: chrono::DateTime<chrono::Utc>,
) -> Result<std::vec::Vec<CohortRegistrationStat>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT 
            CASE 
                WHEN (ae.request_details->'browser_info'->>'is_mobile')::boolean = true THEN 'Mobile'
                WHEN (ae.request_details->'browser_info'->>'is_mobile')::boolean = false THEN 'Desktop'
                ELSE 'Unknown'
            END as device_type,
            COUNT(*) as total_events,
            COUNT(DISTINCT ae.user_id) as unique_users
        FROM analytics_events ae
        INNER JOIN users u ON ae.user_id = u.id
        WHERE ae.event_name = 'user_registration_completed'
            AND ae.timestamp >= $3 AND ae.timestamp < $4
            AND u.created_at >= $1 AND u.created_at < $2
            AND ae.user_id IS NOT NULL
        GROUP BY device_type
        ORDER BY total_events DESC
        "#,
        start_cohort_date,
        end_cohort_date,
        start_event_date,
        end_event_date
    )
    .fetch_all(pool)
    .await?;

    let results = rows
        .into_iter()
        .map(|row| CohortRegistrationStat {
            dimension_name: "Device Type".to_string(),
            dimension_value: row.device_type.unwrap_or_else(|| "Unknown".to_string()),
            total_events: row.total_events.unwrap_or(0),
            unique_users: row.unique_users.unwrap_or(0),
        })
        .collect();

    Ok(results)
}

/// Fetches cohort registration analysis by OS family
#[tracing::instrument(
    name = "query_cohort_registration_by_os",
    skip(pool),
    fields(
        start_cohort_date = %start_cohort_date,
        end_cohort_date = %end_cohort_date,
        start_event_date = %start_event_date,
        end_event_date = %end_event_date
    )
)]
pub async fn query_cohort_registration_by_os(
    pool: &sqlx::PgPool,
    start_cohort_date: chrono::DateTime<chrono::Utc>,
    end_cohort_date: chrono::DateTime<chrono::Utc>,
    start_event_date: chrono::DateTime<chrono::Utc>,
    end_event_date: chrono::DateTime<chrono::Utc>,
) -> Result<std::vec::Vec<CohortRegistrationStat>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT 
            COALESCE(ae.request_details->'browser_info'->>'os_family', 'Unknown') as os_family,
            COUNT(*) as total_events,
            COUNT(DISTINCT ae.user_id) as unique_users
        FROM analytics_events ae
        INNER JOIN users u ON ae.user_id = u.id
        WHERE ae.event_name = 'user_registration_completed'
            AND ae.timestamp >= $3 AND ae.timestamp < $4
            AND u.created_at >= $1 AND u.created_at < $2
            AND ae.user_id IS NOT NULL
        GROUP BY ae.request_details->'browser_info'->>'os_family'
        ORDER BY total_events DESC
        "#,
        start_cohort_date,
        end_cohort_date,
        start_event_date,
        end_event_date
    )
    .fetch_all(pool)
    .await?;

    let results = rows
        .into_iter()
        .map(|row| CohortRegistrationStat {
            dimension_name: "OS Family".to_string(),
            dimension_value: row.os_family.unwrap_or_else(|| "Unknown".to_string()),
            total_events: row.total_events.unwrap_or(0),
            unique_users: row.unique_users.unwrap_or(0),
        })
        .collect();

    Ok(results)
}

/// Fetches cohort registration analysis by browser family
#[tracing::instrument(
    name = "query_cohort_registration_by_browser",
    skip(pool),
    fields(
        start_cohort_date = %start_cohort_date,
        end_cohort_date = %end_cohort_date,
        start_event_date = %start_event_date,
        end_event_date = %end_event_date
    )
)]
pub async fn query_cohort_registration_by_browser(
    pool: &sqlx::PgPool,
    start_cohort_date: chrono::DateTime<chrono::Utc>,
    end_cohort_date: chrono::DateTime<chrono::Utc>,
    start_event_date: chrono::DateTime<chrono::Utc>,
    end_event_date: chrono::DateTime<chrono::Utc>,
) -> Result<std::vec::Vec<CohortRegistrationStat>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT 
            COALESCE(ae.request_details->'browser_info'->>'browser_family', 'Unknown') as browser_family,
            COUNT(*) as total_events,
            COUNT(DISTINCT ae.user_id) as unique_users
        FROM analytics_events ae
        INNER JOIN users u ON ae.user_id = u.id
        WHERE ae.event_name = 'user_registration_completed'
            AND ae.timestamp >= $3 AND ae.timestamp < $4
            AND u.created_at >= $1 AND u.created_at < $2
            AND ae.user_id IS NOT NULL
        GROUP BY ae.request_details->'browser_info'->>'browser_family'
        ORDER BY total_events DESC
        "#,
        start_cohort_date,
        end_cohort_date,
        start_event_date,
        end_event_date
    )
    .fetch_all(pool)
    .await?;

    let results = rows
        .into_iter()
        .map(|row| CohortRegistrationStat {
            dimension_name: "Browser Family".to_string(),
            dimension_value: row.browser_family.unwrap_or_else(|| "Unknown".to_string()),
            total_events: row.total_events.unwrap_or(0),
            unique_users: row.unique_users.unwrap_or(0),
        })
        .collect();

    Ok(results)
}

/// Fetches cohort registration analysis by country (based on IP geolocation)
/// Note: This requires external IP geolocation service for production use
#[tracing::instrument(
    name = "query_cohort_registration_by_country",
    skip(pool),
    fields(
        start_cohort_date = %start_cohort_date,
        end_cohort_date = %end_cohort_date,
        start_event_date = %start_event_date,
        end_event_date = %end_event_date
    )
)]
pub async fn query_cohort_registration_by_country(
    pool: &sqlx::PgPool,
    start_cohort_date: chrono::DateTime<chrono::Utc>,
    end_cohort_date: chrono::DateTime<chrono::Utc>,
    start_event_date: chrono::DateTime<chrono::Utc>,
    end_event_date: chrono::DateTime<chrono::Utc>,
) -> Result<std::vec::Vec<CohortRegistrationStat>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT 
            COALESCE(ae.request_details->>'ip_address', 'Unknown') as ip_address,
            COUNT(*) as total_events,
            COUNT(DISTINCT ae.user_id) as unique_users
        FROM analytics_events ae
        INNER JOIN users u ON ae.user_id = u.id
        WHERE ae.event_name = 'user_registration_completed'
            AND ae.timestamp >= $3 AND ae.timestamp < $4
            AND u.created_at >= $1 AND u.created_at < $2
            AND ae.user_id IS NOT NULL
        GROUP BY ae.request_details->>'ip_address'
        ORDER BY total_events DESC
        "#,
        start_cohort_date,
        end_cohort_date,
        start_event_date,
        end_event_date
    )
    .fetch_all(pool)
    .await?;

    // Use real geolocation service with embedded database (no signup required)
    let geolocation_service = DbIpService::new();
    let mut country_stats: std::collections::HashMap<String, (i64, i64)> = std::collections::HashMap::new();

    for row in rows {
        let ip = row.ip_address.unwrap_or_else(|| "Unknown".to_string());
        let country = geolocation_service.get_country_from_ip(&ip).await.unwrap_or_else(|| "Unknown".to_string());
        
        let entry = country_stats.entry(country).or_insert((0, 0));
        entry.0 += row.total_events.unwrap_or(0);
        entry.1 += row.unique_users.unwrap_or(0);
    }

    let mut results: std::vec::Vec<CohortRegistrationStat> = country_stats
        .into_iter()
        .map(|(country, (total_events, unique_users))| CohortRegistrationStat {
            dimension_name: "Country".to_string(),
            dimension_value: country,
            total_events,
            unique_users,
        })
        .collect();

    // Sort by total events descending
    results.sort_by(|a, b| b.total_events.cmp(&a.total_events));

    Ok(results)
}

/// Comprehensive cohort registration analysis combining all dimensions
#[tracing::instrument(
    name = "query_cohort_registration_comprehensive",
    skip(pool),
    fields(
        start_cohort_date = %start_cohort_date,
        end_cohort_date = %end_cohort_date,
        start_event_date = %start_event_date,
        end_event_date = %end_event_date
    )
)]
pub async fn query_cohort_registration_comprehensive(
    pool: &sqlx::PgPool,
    start_cohort_date: chrono::DateTime<chrono::Utc>,
    end_cohort_date: chrono::DateTime<chrono::Utc>,
    start_event_date: chrono::DateTime<chrono::Utc>,
    end_event_date: chrono::DateTime<chrono::Utc>,
) -> Result<std::vec::Vec<CohortRegistrationStat>, sqlx::Error> {
    let device_stats = query_cohort_registration_by_device(
        pool, start_cohort_date, end_cohort_date, start_event_date, end_event_date
    ).await?;
    
    let os_stats = query_cohort_registration_by_os(
        pool, start_cohort_date, end_cohort_date, start_event_date, end_event_date
    ).await?;
    
    let browser_stats = query_cohort_registration_by_browser(
        pool, start_cohort_date, end_cohort_date, start_event_date, end_event_date
    ).await?;
    
    let country_stats = query_cohort_registration_by_country(
        pool, start_cohort_date, end_cohort_date, start_event_date, end_event_date
    ).await?;

    let mut results = std::vec::Vec::new();
    results.extend(device_stats);
    results.extend(os_stats);
    results.extend(browser_stats);
    results.extend(country_stats);

    Ok(results)
}
