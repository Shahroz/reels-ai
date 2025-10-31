# API Key Rate Limiting Implementation

## Overview

This implementation adds rate limiting to the API key middleware to restrict users to 10 requests per day per IP address. The rate limiting is implemented using in-memory storage without requiring database migrations.

## Implementation Details

### Files Modified/Created

1. **`rate_limit.rs`** - New module containing the rate limiting logic
2. **`auth.rs`** - Modified JWT middleware to include rate limiting
3. **`mod.rs`** - Updated to include the new rate_limit module

### Key Features

- **In-memory storage**: Uses `HashMap` with `RwLock` for thread-safe access
- **Daily reset**: Rate limits reset every 24 hours automatically
- **IP-based tracking**: Each IP address has its own counter
- **Automatic cleanup**: Background task removes expired entries
- **Configurable limits**: Easy to adjust the daily request limit

### Rate Limiting Logic

1. **IP Extraction**: Client IP is extracted using a robust multi-header approach:
   - Checks `x-forwarded-for` (most common proxy header)
   - Checks `x-real-ip` (Nginx proxy)
   - Checks `x-client-ip` (Apache proxy)
   - Checks `cf-connecting-ip` (Cloudflare)
   - Checks `x-forwarded`, `forwarded-for`, `forwarded` (alternative formats)
   - Handles comma-separated IPs and skips private/internal IPs
   - Falls back to connection peer address if no headers found
2. **Rate Check**: Before API key validation, the middleware checks if the IP has exceeded the daily limit
3. **Counter Management**: Each IP gets a counter that tracks:
   - Request count (starts at 0)
   - First request timestamp (null until first request)
   - Last request timestamp (null until first request)
4. **Expiration**: Entries older than 24 hours are automatically cleaned up

### Configuration

The rate limiting can be configured by modifying constants in `rate_limit.rs`:

```rust
pub const RATE_LIMIT_REQUESTS_PER_DAY: u32 = 10;
pub const RATE_LIMIT_CLEANUP_INTERVAL_HOURS: u64 = 24;
```

### Error Response

When rate limit is exceeded, the API returns:

```json
{
  "error": "Rate limit exceeded",
  "message": "Maximum 10 requests per day allowed! Sign up now to get unlimited access to Bounti Studio.",
  "retry_after": "24 hours"
}
```

With HTTP status code `429 Too Many Requests`.

### Usage

The rate limiting is automatically applied to all API routes that use the `JwtMiddleware`. No changes are required to existing route configurations.

### Third-Party Integration Support

The enhanced IP extraction is specifically designed to work with third-party services making AJAX calls from JavaScript applications:

- **Proxy Support**: Automatically detects and uses proxy headers like `x-forwarded-for`
- **CDN Support**: Works with Cloudflare (`cf-connecting-ip`) and other CDNs
- **Load Balancer Support**: Handles various load balancer configurations
- **Private IP Filtering**: Skips internal/proxy IPs to get the real client IP
- **Multiple IP Handling**: Correctly parses comma-separated IP lists

This ensures accurate rate limiting even when requests come through:
- Corporate firewalls and proxies
- Content delivery networks (CDNs)
- Load balancers and reverse proxies
- Cloud hosting platforms

### Testing

The implementation includes comprehensive unit tests that verify:
- Basic rate limiting functionality (10 requests allowed, 11th rejected)
- Different IP addresses have separate counters
- Unknown IP handling
- Counter management

### Performance Considerations

- **Memory usage**: Minimal overhead with automatic cleanup
- **Thread safety**: Uses `RwLock` for concurrent access
- **Cleanup**: Background task prevents memory leaks
- **No database impact**: Completely in-memory solution

### Future Enhancements

Potential improvements could include:
- Redis-based storage for distributed systems
- Different rate limits for different user types
- Sliding window rate limiting
- Rate limit headers in responses
- Admin endpoints to view/manage rate limits
