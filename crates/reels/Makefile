# Makefile for project tasks

########################################
# Environment & Configuration Variables
########################################
ENV_FILE = .env
-include $(ENV_FILE)

DATABASE_URL ?= postgresql://localuser:localpassword@localhost:5447/localdatabase

# GCS Emulator Configuration
GCS_EMULATOR_IMAGE = fsouza/fake-gcs-server:latest
GCS_EMULATOR_CONTAINER = gcs-emulator
GCS_EMULATOR_PORT ?= 4443
GCS_EMULATOR_DATA_DIR = ./test-data/gcs

# Flag for optional emulator usage in dev/app commands
EMULATOR ?= false

$(shell mkdir -p $(GCS_EMULATOR_DATA_DIR))

########################################
# GCS Emulator Commands
########################################
.PHONY: gcs-start gcs-stop gcs-logs gcs-clean gcs-reset gcs-status clean-gcs

gcs-start:
	@echo "Starting GCS emulator container '$(GCS_EMULATOR_CONTAINER)'..."
	@if docker ps --filter "name=$(GCS_EMULATOR_CONTAINER)" --filter "status=running" | grep -q $(GCS_EMULATOR_CONTAINER); then \
		echo "✅ GCS emulator container '$(GCS_EMULATOR_CONTAINER)' is already running"; \
	else \
		docker run --name $(GCS_EMULATOR_CONTAINER) \
			-p $(GCS_EMULATOR_PORT):4443 \
			-v $(abspath $(GCS_EMULATOR_DATA_DIR)):/data \
			-e FAKE_GCS_SERVER_HOST=0.0.0.0 \
			-e FAKE_GCS_SERVER_PORT=4443 \
			-d $(GCS_EMULATOR_IMAGE) \
			-scheme http \
			-port 4443 \
			-public-host localhost:$(GCS_EMULATOR_PORT) 2>/dev/null || \
		(docker start $(GCS_EMULATOR_CONTAINER) 2>/dev/null && echo "✅ Started existing GCS emulator container '$(GCS_EMULATOR_CONTAINER)'") || \
		(echo "❌ Failed to start GCS emulator container" && exit 1); \
	fi

gcs-stop:
	@echo "Stopping and removing GCS emulator container '$(GCS_EMULATOR_CONTAINER)'..."
	@docker stop $(GCS_EMULATOR_CONTAINER) || true
	@docker rm $(GCS_EMULATOR_CONTAINER) || true
	@echo "GCS emulator container stopped and removed."

gcs-logs:
	@echo "Showing logs for GCS emulator container '$(GCS_EMULATOR_CONTAINER)'..."
	@docker logs -f $(GCS_EMULATOR_CONTAINER)

gcs-clean:
	@echo "Removing GCS emulator data directory $(GCS_EMULATOR_DATA_DIR)..."
	@rm -rf $(GCS_EMULATOR_DATA_DIR)
	@mkdir -p $(GCS_EMULATOR_DATA_DIR)
	@echo "GCS emulator data directory cleaned and recreated."

gcs-reset: gcs-stop gcs-clean gcs-start
	@echo "GCS emulator reset complete (container stopped, data removed, container restarted)."

gcs-status:
	@echo "Checking GCS emulator status..."
	@if docker ps --filter "name=$(GCS_EMULATOR_CONTAINER)" --filter "status=running" | grep -q $(GCS_EMULATOR_CONTAINER); then \
		echo "✅ GCS emulator is running on port $(GCS_EMULATOR_PORT)"; \
		echo "Testing endpoint..."; \
		curl -s "http://localhost:$(GCS_EMULATOR_PORT)/storage/v1/b" > /dev/null && \
		echo "✅ GCS emulator endpoint is responding" || \
		echo "❌ GCS emulator endpoint is not responding"; \
	else \
		echo "❌ GCS emulator is not running"; \
	fi

clean-gcs:
	@echo "Stopping and removing GCS emulator container '$(GCS_EMULATOR_CONTAINER)'..."
	@docker stop $(GCS_EMULATOR_CONTAINER) || true
	@docker rm $(GCS_EMULATOR_CONTAINER) || true
	@echo "GCS emulator cleanup finished."

run: app-run

# Set emulator environment if EMULATOR=true flag is provided
EMULATOR_ENV =
BASE_ENV = DATABASE_URL=$(DATABASE_URL) APP_ENV=dev
ifeq ($(EMULATOR),true)
	EMULATOR_ENV = $(BASE_ENV) STORAGE_EMULATOR_HOST=http://localhost:$(GCS_EMULATOR_PORT)
	EMULATOR_DEPS = gcs-start
else
	EMULATOR_ENV = $(BASE_ENV)
	EMULATOR_DEPS = 
endif

app-run: $(EMULATOR_DEPS)
	@echo "Running app..."
	@$(MAKE) kill-8081
	cd backend && env $(EMULATOR_ENV) cargo run



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
.PHONY: dev gcs-setup-dev-buckets

gcs-setup-dev-buckets:
	@echo "Setting up GCS buckets in emulator..."
	@sleep 2  # Give emulator time to fully start
	@curl -s -X POST "http://localhost:$(GCS_EMULATOR_PORT)/storage/v1/b?project=test-project" \
		-H "Content-Type: application/json" \
		-d '{"name": "bounti_prod_narrativ_public"}' > /dev/null 2>&1 || true
	@echo "✅ GCS bucket 'bounti_prod_narrativ_public' ready in emulator"

dev:
	@echo "Starting local development environment..."
	@$(MAKE) kill-8080
	@echo "Starting backend..."
	(cd backend && env DATABASE_URL=$(DATABASE_URL) APP_ENV=dev cargo run &) && sleep 5


########################################
# Help and Documentation
########################################
.PHONY: help

help:
	@echo "Available targets:"
	@echo "  GCS Emulator:"
	@echo "    gcs-start          - Start GCS emulator container"
	@echo "    gcs-stop           - Stop GCS emulator container"
	@echo "    gcs-logs           - Show GCS emulator container logs"
	@echo "    gcs-clean          - Remove GCS emulator data directory"
	@echo "    gcs-reset          - Reset GCS emulator (stop, clean, start)"
	@echo "    gcs-status         - Check GCS emulator status and health"
	@echo "    clean-gcs          - Stop and remove GCS emulator container"
	@echo ""
	@echo "  Development:"
	@echo "    dev                - Start local development environment"
	@echo "    app-run            - Run backend application"
	@echo "    run                - Run backend application"
	@echo ""
	@echo "  Docker:"
	@echo "    docker-build       - Build Docker image for the main app"
	@echo "    docker-run         - Run Docker container on port 8081"
	@echo "    docker-clean       - Remove stopped containers from the app image"
	@echo ""
	@echo "  Utilities:"
	@echo "    kill-8080          - Kill process using port 8080"
	@echo "    kill-8081          - Kill process using port 8081"
	@echo "    gcs-setup-dev-buckets - Setup GCS buckets in emulator for development"
	@echo ""
	@echo "  Configuration Options:"
	@echo "    EMULATOR=true      - Use GCS emulator (for dev/app-run commands)"
	@echo ""
	@echo "  Examples:"
	@echo "    make dev EMULATOR=true"

########################################
# Docker Commands
########################################
.PHONY: docker-build docker-run docker-clean

# Build the Docker image for the main app
docker-build:
	docker build -t narrativ_app .

# Run the Docker container mapping port 8081
docker-run:
	docker run --env-file .env -p 8081:8081 narrativ_app

# Optionally remove stopped containers from this image
docker-clean:
	-docker rm $$(docker ps -a -q --filter "ancestor=narrativ_app") || true