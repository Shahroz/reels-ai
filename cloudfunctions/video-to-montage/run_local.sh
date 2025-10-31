#!/bin/bash
#
# Runs the video-to-montage cloud function locally using functions-framework.
#
# Pre-requisites:
# 1. Python 3.11+ installed.
# 2. gcloud CLI installed and authenticated (run `gcloud auth application-default login`).
# 3. Dependencies installed (run `pip install -r requirements.txt`).

echo "Starting local server for generate_montage_json function..."
echo "The server will be available at http://localhost:8080"
echo "Press CTRL+C to stop."
export GOOGLE_CLOUD_PROJECT="bounti-prod-322900"
export GOOGLE_CLOUD_LOCATION="us-central1"
#export GOOGLE_GENAI_USE_VERTEXAI=True
export GCP_SERVICE_ACCOUNT_PATH=/Users/pawel/.bounti/service_account.json
export GOOGLE_APPLICATION_CREDENTIALS=/Users/pawel/.bounti/service_account.json

functions-framework --target=generate_montage_json --source=main.py --debug