# New Relic OpenTelemetry Troubleshooting Guide

## Service Not Appearing in New Relic UI

If you've run the validation script but don't see your service name in New Relic, try these troubleshooting steps:

### 1. Verify Configuration
Run the validation script and check the configuration output:
```bash
cargo run --bin phase1_error_validation --features newrelic
```

Look for:
- ✅ **Service Name**: Should show "real-estate-local" (or your chosen name)
- ✅ **OTLP Endpoint**: Should show "http://otlp.nr-data.net:4318"
- ✅ **License Key**: Should show "✅ Set"

### 2. Check New Relic Account Region  
Make sure your New Relic account region matches the endpoint:
- **US accounts**: `https://otlp.nr-data.net:4318` 
- **EU accounts**: `https://otlp.eu01.nr-data.net:4318`
- **FedRAMP accounts**: `https://gov-otlp.nr-data.net:4318`

**Important**: If you see "scheme is not http" error, try using `https://` instead of `http://` in your `.env` file.

### 3. Verify License Key
Your license key should be an "Ingest - License" key, not:
- User API Key
- Query API Key
- Browser monitoring key

Find it at: [New Relic API Keys](https://one.newrelic.com/launcher/api-keys-ui.api-keys-launcher)

### 4. Where to Look in New Relic UI

1. **APM & Services**: https://one.newrelic.com/apm
   - Look for service named "real-estate-local"
   - May take 1-2 minutes to appear

2. **Distributed Tracing**: https://one.newrelic.com/distributed-tracing
   - Search for traces with your service name

3. **Errors Inbox**: https://one.newrelic.com/errors-inbox
   - Look for errors from your service

4. **Logs**: https://one.newrelic.com/logger
   - Search for `service.name:real-estate-local`

### 5. Data Delay
- OpenTelemetry data can take **1-5 minutes** to appear in New Relic
- Run the script and wait a few minutes before checking
- The script now waits 15 seconds for export, but UI delays are separate

### 6. Debug Information
The enhanced validation script now shows:
- Exact service name being sent
- OTLP endpoint being used
- License key presence verification
- Direct links to New Relic UI

### 7. Alternative: Check Raw Data
If the service doesn't appear in APM, try searching in NRQL:
```sql
SELECT * FROM Span WHERE service.name = 'real-estate-local' SINCE 1 hour ago
```

Run this query at: https://one.newrelic.com/data-exploration/query-builder

### 8. Firewall/Network Issues
Ensure your network allows outbound HTTP connections to:
- `otlp.nr-data.net` on port `4318`
- Some corporate firewalls block this

### 9. Verbose Logging
Set environment variable for more detailed logs:
```bash
export RUST_LOG=debug
cargo run --bin phase1_error_validation --features newrelic
```

This will show detailed OpenTelemetry export information.