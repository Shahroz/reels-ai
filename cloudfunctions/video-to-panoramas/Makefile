# Makefile for video-to-panoramas Cloud Function

# --- Configuration ---
# GCloud settings
PROJECT_ID ?= $(shell gcloud config get-value project)
REGION ?= us-central1
FUNCTION_NAME ?= extract_pans

# Docker settings
IMAGE_NAME ?= gcr.io/$(PROJECT_ID)/$(FUNCTION_NAME)
TAG ?= latest

# Local test settings
# Example: make test TEST_VIDEO=./my_video.mp4
TEST_VIDEO ?=
OUTPUT_DIR ?= output

.PHONY: help install test build run deploy clean

help:
	@echo "Makefile for video-to-panoramas Cloud Function"
	@echo ""
	@echo "Usage:"
	@echo "  make install         - Install Python dependencies from requirements.txt"
	@echo "  make test            - Run local test script. Requires TEST_VIDEO to be set."
	@echo "  make build           - Build the Docker image for the function."
	@echo "  make run             - Run the Docker container locally on port 8080."
	@echo "  make deploy          - Deploy the function to Google Cloud."
	@echo "  make clean           - Remove local output directory and Python cache files."
	@echo ""
	@echo "Variables:"
	@echo "  PROJECT_ID       - Your Google Cloud project ID (default: current gcloud config)."
	@echo "  REGION           - Google Cloud region (default: us-central1)."
	@echo "  FUNCTION_NAME    - Cloud Function name (default: extract_pans)."
	@echo "  IMAGE_NAME       - Docker image name (default: gcr.io/PROJECT_ID/FUNCTION_NAME)."
	@echo "  TEST_VIDEO       - (Required for 'test') Path to a video file. Example: make test TEST_VIDEO=path/to/video.mp4"
	@echo "  OUTPUT_DIR       - Output directory for 'make test' (default: output)."

install: requirements.txt
	pip install -r requirements.txt

test:
	@if [ -z "$(TEST_VIDEO)" ]; then \
		echo "Error: TEST_VIDEO variable is not set."; \
		echo "Usage: make test TEST_VIDEO=/path/to/your/video.mp4"; \
		exit 1; \
	fi
	@if [ ! -f "$(TEST_VIDEO)" ]; then \
		echo "Error: Test video not found at '$(TEST_VIDEO)'"; \
		exit 1; \
	fi
	python test_local.py $(TEST_VIDEO) --output_prefix $(OUTPUT_DIR) --stitch

build: Dockerfile
	docker build -t $(IMAGE_NAME):$(TAG) .

run: build
	docker run --rm -p 8080:8080 $(IMAGE_NAME):$(TAG)

deploy:
	@echo "Deploying function '$(FUNCTION_NAME)' to project '$(PROJECT_ID)' in region '$(REGION)'..."
	gcloud functions deploy $(FUNCTION_NAME) \
		--gen2 \
		--runtime=python311 \
		--region=$(REGION) \
		--source=. \
		--entry-point=extract_pans \
		--trigger-http \
		--allow-unauthenticated

clean:
	@echo "Cleaning output and cache files..."
	rm -rf $(OUTPUT_DIR)
	rm -rf .pytest_cache
	find . -type d -name "__pycache__" -exec rm -rf {} +
	find . -type f -name "*.pyc" -delete