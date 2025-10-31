//! Detects if a User-Agent string indicates a mobile device.
//!
//! This function analyzes HTTP User-Agent headers to determine if the request
//! comes from a mobile device. Used for device-aware redirect logic in 
//! authentication flows to serve appropriate studio experiences.

/// Detects if a User-Agent string indicates a mobile device
pub fn is_mobile_device(user_agent: Option<&str>) -> bool {
    match user_agent {
        Some(ua) => {
            let ua_lower = ua.to_lowercase();
            // Check for mobile indicators in User-Agent string
            ua_lower.contains("mobile") 
                || ua_lower.contains("android") 
                || ua_lower.contains("iphone") 
                || ua_lower.contains("ipad")
                || ua_lower.contains("ipod")
                || ua_lower.contains("blackberry")
                || ua_lower.contains("windows phone")
                || ua_lower.contains("opera mini")
                || ua_lower.contains("opera mobi")
        }
        None => {
            // Default to desktop if no User-Agent is provided
            false
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_mobile_user_agents() {
        // Test various mobile User-Agent strings
        let mobile_uas = [
            "Mozilla/5.0 (iPhone; CPU iPhone OS 14_7_1 like Mac OS X)",
            "Mozilla/5.0 (Linux; Android 11; Pixel 5)",
            "Mozilla/5.0 (iPad; CPU OS 14_7_1 like Mac OS X)",
            "Mozilla/5.0 (BlackBerry; U; BlackBerry 9800)",
            "Mozilla/5.0 (compatible; MSIE 10.0; Windows Phone 8.0)",
            "Opera/9.80 (J2ME/MIDP; Opera Mini/9.80)",
        ];

        for ua in mobile_uas.iter() {
            assert!(
                super::is_mobile_device(Some(ua)),
                "Should detect {} as mobile", ua
            );
        }
    }

    #[test]
    fn test_desktop_user_agents() {
        // Test desktop User-Agent strings
        let desktop_uas = [
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36",
        ];

        for ua in desktop_uas.iter() {
            assert!(
                !super::is_mobile_device(Some(ua)),
                "Should detect {} as desktop", ua
            );
        }
    }

    #[test]
    fn test_no_user_agent() {
        // Test behavior when no User-Agent is provided
        assert!(!super::is_mobile_device(None));
    }
}