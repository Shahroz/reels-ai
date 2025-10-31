# New Relic Observability Validation Scripts

## Overview

This directory contains validation scripts to test and verify New Relic observability enhancements at each phase of implementation. Each script generates specific scenarios to validate that New Relic receives the expected data with proper context and attribution.

## Prerequisites

1. **New Relic Account**: Ensure you have a New Relic account with appropriate permissions
2. **New Relic License Key**: Set the `NEW_RELIC_LICENSE_KEY` environment variable
3. **Rust Environment**: Rust 1.70+ with cargo installed
4. **Feature Flag**: Build with `--features newrelic` to enable telemetry

## Environment Setup

The validation scripts automatically load environment variables from a `.env` file using dotenvy, just like the main application.

### Option 1: Create a `.env` file (Recommended)

Create a `.env` file in the `validation_scripts/` directory:

```bash
# .env file for validation scripts
NEW_RELIC_LICENSE_KEY=your_license_key_here
OTEL_SERVICE_NAME=narrativ-validation
NEW_RELIC_OTLP_ENDPOINT=https://otlp.nr-data.net
OTEL_EXPORTER_OTLP_COMPRESSION=gzip
OTEL_ATTRIBUTE_VALUE_LENGTH_LIMIT=4095
```

**Where to find your license key**:
1. Go to [one.newrelic.com](https://one.newrelic.com)
2. Click account settings (user icon in top right)
3. Go to **API keys**
4. Find **Ingest - License** keys and copy the value

### Option 2: Set environment variables manually

```bash
# Set New Relic license key
export NEW_RELIC_LICENSE_KEY="your_license_key_here"

# Set service name (how it appears in New Relic)
export OTEL_SERVICE_NAME="narrativ-validation"

# Optional: Set New Relic endpoint (defaults to US)
export NEW_RELIC_OTLP_ENDPOINT="https://otlp.nr-data.net"

# Optional: Enable compression (recommended)
export OTEL_EXPORTER_OTLP_COMPRESSION="gzip"
```

## Validation Scripts

### Continuous Validation (Recommended for Troubleshooting)

**Script**: `continuous_validation.rs`
**Purpose**: Continuously sends telemetry data until canceled (Ctrl+C)

**What it does**:
- Sends a variety of spans every 5 seconds
- Includes success operations, errors, database queries, user actions, and business metrics
- Shows real-time configuration and statistics
- Perfect for debugging connectivity issues
- Allows you to monitor New Relic UI while data is being sent

**Run**:
```bash
cd validation_scripts
cargo run --bin continuous_validation --features newrelic
```

**Expected New Relic Results**:
- Service appears in APM & Services within 1-2 minutes
- Continuous stream of traces in Distributed Tracing
- Mix of successful and error spans
- Real-time feedback on data flow

Press `Ctrl+C` to stop the continuous validation.

## Phase-Specific Validation

### Phase 1: Error Attribution & Span Enrichment

**Script**: `phase1_error_validation.rs`
**Purpose**: Validates enhanced error handling middleware

**What it tests**:
- HTTP client errors (4xx) with user context
- HTTP server errors (5xx) with detailed attribution
- Authenticated user permission errors
- Request correlation and trace propagation

**Run**:
```bash
cd validation_scripts
cargo run --bin phase1_error_validation --features newrelic
```

**Expected New Relic Results**:
- Errors appear in New Relic Errors Inbox
- Error spans contain user context and request IDs
- Distributed traces show error attribution
- Request correlation links related operations

### Phase 2: Database Query Instrumentation  

**Script**: `phase2_database_validation.rs`
**Purpose**: Validates database query performance tracking

**What it tests**:
- Fast query performance metrics
- Slow query detection and alerting
- Database error attribution with query context
- Connection pool health metrics

**Run**:
```bash
cargo run --bin phase2_database_validation --features newrelic
```

**Expected New Relic Results**:
- Database queries visible in distributed traces
- Query performance metrics (duration, row counts)
- Slow query alerts and identification
- Database error context with query information

### Phase 3: Custom Business Events

**Script**: `phase3_events_validation.rs`  
**Purpose**: Validates custom business events and metrics

**What it tests**:
- User registration and lifecycle events
- Document creation and AI interaction metrics
- Conversion funnel tracking
- Business KPI and performance metrics

**Run**:
```bash
cargo run --bin phase3_events_validation --features newrelic
```

**Expected New Relic Results**:
- Custom events queryable via NRQL
- Business metrics in dashboards
- User journey and conversion data
- Performance KPI metrics for monitoring

## Validation Workflow

### 1. Pre-Validation Setup
```bash
# Navigate to validation scripts directory
cd /path/to/validation_scripts

# Ensure dependencies are available
cargo check --features newrelic

# Set required environment variables
export NEW_RELIC_LICENSE_KEY="your_key"
```

### 2. Run Phase-Specific Validation
```bash
# Run the validation script for the current phase
cargo run --bin phase1_error_validation --features newrelic

# Wait 2-3 minutes for data to appear in New Relic
sleep 180
```

### 3. Manual New Relic Verification

**For Errors Inbox**:
1. Navigate to New Relic → APM → Errors Inbox
2. Filter by app name `narrativ-validation` 
3. Verify errors contain expected context attributes
4. Check error grouping and stack traces

**For Distributed Tracing**:
1. Navigate to New Relic → APM → Distributed Tracing
2. Search for traces containing validation scenarios
3. Verify span hierarchy and attributes
4. Check trace duration and error attribution

**For Custom Events (NRQL)**:
```sql
-- Query custom events
SELECT * FROM UserRegistration WHERE appName = 'narrativ-validation' SINCE 1 hour ago

-- Query business metrics  
SELECT * FROM BusinessMetrics WHERE appName = 'narrativ-validation' SINCE 1 hour ago

-- Query performance data
SELECT average(duration) FROM Span WHERE appName = 'narrativ-validation' SINCE 1 hour ago
```

### 4. Validation Checklist

For each phase, verify:
- [ ] Data appears in New Relic within 2-3 minutes
- [ ] All expected attributes are present and accurate
- [ ] Error attribution works correctly
- [ ] Performance data is reasonable and actionable
- [ ] No missing spans or uninstrumented time
- [ ] Custom events are queryable via NRQL

## Troubleshooting

### Data Not Appearing in New Relic

1. **Check License Key**: Verify `NEW_RELIC_LICENSE_KEY` is correct
2. **Check Network**: Ensure outbound HTTPS to New Relic is allowed
3. **Check Logs**: Look for OpenTelemetry export errors in script output
4. **Wait Longer**: New Relic can take 2-5 minutes to show data
5. **Check App Name**: Verify you're filtering by the correct app name

### Compilation Errors

1. **Check Dependencies**: Ensure `observability` crate is built with `newrelic` feature
2. **Update Rust**: Ensure Rust version 1.70+
3. **Clean Build**: Run `cargo clean && cargo build --features newrelic`

### Missing Context in New Relic

1. **Check Span Attributes**: Verify spans contain expected custom attributes
2. **Check Feature Flags**: Ensure `newrelic` feature is enabled during build
3. **Check Trace Propagation**: Verify parent-child span relationships

## Next Steps

After successful validation of each phase:

1. **Document Results**: Record validation outcomes in the observability plan
2. **Production Deployment**: Apply validated changes to production environment  
3. **Monitor Performance**: Watch for any performance impact from new instrumentation
4. **Create Alerts**: Set up New Relic alerts based on validated metrics
5. **Build Dashboards**: Create dashboards using validated custom events and metrics

## Contributing

When adding new validation scenarios:

1. Follow the existing script structure and patterns
2. Include comprehensive test coverage for new scenarios
3. Update this README with new validation procedures
4. Ensure scripts follow coding guidelines (one function per test scenario)
5. Add appropriate error handling and logging