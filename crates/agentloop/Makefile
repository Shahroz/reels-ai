# Makefile for managing the custom PostgreSQL database and project tasks

########################################
# Environment & Configuration Variables
########################################
ENV_FILE = .env
-include $(ENV_FILE)
export $(shell sed 's/=.*//' $(ENV_FILE))

POSTGRES_USER ?= localuser
POSTGRES_PASSWORD ?= localpassword
POSTGRES_DB ?= localdatabase
POSTGRES_PORT ?= 5447
DATABASE_URL ?= postgresql://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@localhost:$(POSTGRES_PORT)/$(POSTGRES_DB)

IMAGE_NAME = styleclone-postgres-db
CONTAINER_NAME = styleclone-postgres-container
PGDATA_DIR = ./pgdata

$(shell mkdir -p $(PGDATA_DIR))

########################################
# Database and Docker Commands
########################################
.PHONY: db-build db-start db-stop db-connect db-reset db-logs db-clean db-url db-migrate clean-docker

db-build:
	@echo "Building custom PostgreSQL image '$(IMAGE_NAME)' from DockerfileDB..."
	@docker build -t $(IMAGE_NAME) -f DockerfileDB .

db-start:
	@echo "Starting custom PostgreSQL container '$(CONTAINER_NAME)'..."
	@docker run --name $(CONTAINER_NAME) \
		-e POSTGRES_PASSWORD=$(POSTGRES_PASSWORD) \
		-e POSTGRES_USER=$(POSTGRES_USER) \
		-e POSTGRES_DB=$(POSTGRES_DB) \
		-p $(POSTGRES_PORT):5432 \
		-v $(abspath $(PGDATA_DIR)):/var/lib/postgresql/data \
		-d $(IMAGE_NAME)
	@echo "Custom PostgreSQL container started. Use 'make db-logs' or 'make db-connect'."

db-stop:
	@echo "Stopping and removing custom PostgreSQL container '$(CONTAINER_NAME)'..."
	@docker stop $(CONTAINER_NAME) || true
	@docker rm $(CONTAINER_NAME) || true
	@echo "Container stopped and removed."

db-connect:
	@echo "Connecting to database $(POSTGRES_DB) in container $(CONTAINER_NAME)..."
	@docker exec -it $(CONTAINER_NAME) psql -U $(POSTGRES_USER) -d $(POSTGRES_DB)

db-reset: db-stop db-clean db-start
	@echo "Database reset complete (container stopped, data removed, container restarted)."

db-logs:
	@echo "Showing logs for container $(CONTAINER_NAME)..."
	@docker logs -f $(CONTAINER_NAME)

db-clean:
	@echo "Removing local data directory $(PGDATA_DIR)..."
	@rm -rf $(PGDATA_DIR)
	@mkdir -p $(PGDATA_DIR)
	@echo "Local data directory cleaned and recreated."

db-url:
	@echo "Database connection URL:"
	@echo "$(DATABASE_URL)"

db-migrate:
	@echo "Running migrations"
	@DATABASE_URL=$(DATABASE_URL) sqlx migrate run

clean-docker:
	@echo "Stopping and removing container $(CONTAINER_NAME)..."
	@docker stop $(CONTAINER_NAME) || true
	@docker rm $(CONTAINER_NAME) || true
	@echo "Removing image $(IMAGE_NAME)..."
	@docker rmi $(IMAGE_NAME) || true
	@echo "Docker cleanup finished."

########################################
# Testing and API Client Generation
########################################
.PHONY: test test-frontend test-backend

test: test-frontend test-backend

test-frontend:
	@echo "Running frontend tests..."
	cd frontend && npm run build

test-backend:
	@echo "Running backend tests..."
	cargo test

run: db-migrate dump-db-schema generate-api-client test-frontend test-backend app-run

app-run:
	@echo "Running app..."
	@$(MAKE) kill-8081
	cd backend && cargo run

dump-db-schema:
	@echo "Dumping database schema to backend/db_schema.sql..."
	@mkdir -p backend
	@docker exec -e PGPASSWORD=$(POSTGRES_PASSWORD) $(CONTAINER_NAME) pg_dump --schema-only -U $(POSTGRES_USER) -d $(POSTGRES_DB) > backend/db_schema.sql
	@echo "Database schema dump complete."

generate-api-client:
	@$(MAKE) kill-8081
	@echo "Starting backend server in background..."
	PORT=8081 cargo run --bin agentloop & SERVER_PID=$$!; \
	echo "Waiting 5s for server to start (PID: $$SERVER_PID)..."; \
	sleep 20; \
	echo "Generating frontend API client..."; \
 	# Generate full TypeScript client using fetch-based code generator
 	# In one conditional: clean, generate, catch errors
	if cd frontend && ( \
		echo "Cleaning existing API client files..." && \
		rm -rf src/api/* && \
		echo "Generating full TypeScript client..." && \
		npx openapi-typescript-codegen --input http://127.0.0.1:8081/api-docs/openapi.json --output src/api --client fetch --useOptions \
		echo "Generating full TypeScript client for narrativ ..." \
		npx openapi-typescript-codegen --input http://127.0.0.1:8081/api-docs/openapi.json --output ../../narrativ/frontend/src/loupe_api --client fetch --useOptions \
	); then \
		echo "API client generation successful."; \
	else \
		echo "API client generation failed."; \
		echo "Stopping backend server (PID: $$SERVER_PID)..."; \
		kill $$SERVER_PID; exit 1; \
	fi; \
	echo "Stopping backend server (PID: $$SERVER_PID)..."; \
	kill $$SERVER_PID; \
	echo "Done."

kill-8080:
	@echo "Killing process on port 8080..."
	@PID=$$(lsof -ti tcp:8080); \
	if [ ! -z "$$PID" ]; then \
		kill -9 $$PID; \
		echo "Killed process on port 8080."; \
	else \
		echo "No process is using port 8080."; \
	fi

kill-8081:
	@echo "Killing process on port 8081..."
	@PID=$$(lsof -ti tcp:8081); \
	if [ ! -z "$$PID" ]; then \
		kill -9 $$PID; \
		echo "Killed process on port 8081."; \
	else \
		echo "No process is using port 8081."; \
	fi

########################################
# Local Development Target
########################################
.PHONY: dev

dev:
	@echo "Starting local development environment..."
	@#$(MAKE) test
	#@#$(MAKE) generate-api-client
	@$(MAKE) dump-db-schema
	@echo "Starting backend..."
	(cargo run --bin agentloop &) && sleep 5
	@echo "Starting frontend..."
	cd frontend && npm run dev

# End of Makefile

# Makefile

.PHONY: docker-build docker-run docker-clean

# Build the Docker image for the main app
docker-build:
	docker build -t narrativ_app .

# Run the Docker container mapping port 8080
docker-run:
	docker run --env-file .env -p 8080:8080 narrativ_app

# Optionally remove stopped containers from this image
docker-clean:
	-docker rm $$(docker ps -a -q --filter "ancestor=narrativ_app") || true
