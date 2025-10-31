# Developer Documentation - reels_app 

## Overview
This project is a Website Style Cloning App built using Rust for the backend.
This document provides instructions for setting up the project, running the database, and executing the application.

## Prerequisites
- Rust (stable version)
- Docker and Docker Compose
- Make

## Quick Start
```bash
# 1. Start development environment
make dev

# 2. Run tests
make test-backend

# 3. For Docker development
make docker-dev
```

## Setup and Build Instructions

### Database Setup
To build and start the database container, use the following commands:
- **Build DB Image:** `make db-build`
- **Start DB Container:** `make db-start`
- **Reset the Database:** `make db-reset`
- **View DB Logs:** `make db-logs`

### Running the Application

#### Local Development (Native)
For daily development with fast iteration:
- **Run Application:** `make dev` (starts backend in development mode)

#### Docker Development (Production-like)
For testing production builds locally:
- **Development Mode:** `make docker-dev` (permissive CORS, debug logging)
- **Production Mode:** `make docker-prod-like` (restrictive CORS, production logging)

#### Manual Docker Commands
- **Build Docker Image:** `make docker-build` (local, no New Relic)
- **Run Development Container:** `make docker-run-dev`
- **Run Production Container:** `make docker-run-prod`
- **View Container Logs:** `make docker-logs`
- **Stop Container:** `make docker-stop`

### Testing

#### Quick Testing Commands
- **Run All Tests:** `make test` (backend)
- **Run Backend Tests:** `make test-backend` (local, fast)

#### Advanced Testing Options
- **Docker Testing:** `make test-backend-docker` (matches CI exactly)
- **Manual Test Environment:** `make test-env-start` / `make test-env-stop`
- **Check Service Status:** `make status`

#### Testing with GCS Emulator
The backend assets tests require a GCS emulator for integration testing. The test system automatically:
1. Starts PostgreSQL and GCS emulator services
2. Runs all backend tests with proper environment variables
3. Ensures test isolation with unique prefixes per test
4. Cleans up all test artifacts automatically

**Testing Options:**

*Option 1: Local Development (Faster)*
```bash
# Run backend tests (automatically starts required services)
make test-backend

# Manually start/stop test services
make test-env-start
make test-env-stop

# Check service status
make status
```

*Option 2: Docker (Matches CI)*
```bash
# Run tests in exact same environment as CI
make test-backend-docker

# Or manually with docker-compose
docker-compose -f docker-compose.test.yml up --build --abort-on-container-exit --exit-code-from test
```

**When to use each:**
- **Local (`make test-backend`)**: Faster, easier debugging, daily development
- **Docker (`make test-backend-docker`)**: CI troubleshooting, environment consistency, pre-commit verification

#### Development Approach Comparison

| Approach | Use Case | Pros | Cons |
|----------|----------|------|------|
| `make dev` | Daily development | ✅ Fast iteration<br/>✅ Easy debugging<br/>✅ Hot reload | ⚠️ Requires local Rust setup |
| `make docker-dev` | Production debugging | ✅ Production-like build<br/>✅ Consistent environment<br/>✅ Easy CORS testing | ⚠️ Slower builds<br/>⚠️ No hot reload |
| `make docker-prod-like` | Bug reproduction | ✅ Exact production behavior<br/>✅ Restrictive CORS<br/>✅ Production logging | ⚠️ Less developer-friendly |


## Development Guidelines

### Code Standards
- Follow the instructions in `CODING_GUIDELINES.md` and refer to `BRIEF.md` for project understanding
- Use `make help` to see all available commands

### Backend Development
- Run commands inside the `backend` folder: `cd backend && cargo test`
- Use `make test-backend` for fast iteration during development
- Database schema changes: `make dump-db-schema` before committing


### Docker Development
- Use `make docker-dev` for production-like testing with development-friendly settings
- Use `make docker-prod-like` to reproduce production issues locally
- Check logs with `make docker-logs`

### Testing Strategy
1. **Daily development**: `make test-backend` (fast, local)
2. **Pre-commit**: `make test` (full test suite)
3. **CI debugging**: `make test-backend-docker` (exact CI environment)

### Service Management
- Start services: `make test-env-start` 
- Check status: `make status`
- Stop services: `make test-env-stop`

### Common Workflows

**New Feature Development:**
```bash
make dev                    # Start development environment
make test-backend          # Run tests during development
make docker-dev            # Test production build locally
```

**Bug Investigation:**
```bash
make docker-prod-like      # Reproduce production environment
make docker-logs           # Check application logs
make status                # Verify service health
```

Happy coding! 🚀
