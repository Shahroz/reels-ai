# Video to Montage Cloud Function

This directory contains a Google Cloud Function that generates a video montage JSON definition based on video and photo assets.

## Deployment

### Prerequisites
1. Install Google Cloud CLI: https://cloud.google.com/sdk/docs/install
2. Authenticate: `gcloud auth login`
3. Set your project: `gcloud config set project YOUR_PROJECT_ID`
4. Enable required APIs:
   ```bash
   gcloud services enable cloudfunctions.googleapis.com
   gcloud services enable cloudbuild.googleapis.com
   gcloud services enable storage.googleapis.com
   ```

### Deploy the Function (2nd Gen)

```bash
gcloud functions deploy generate_montage \
  --gen2 \
  --runtime=python311 \
  --region=us-central1 \
  --entry-point=generate_montage_json \
  --source=. \
  --trigger-http \
  --allow-unauthenticated
```
(Adjust region, auth policy, and project as needed. You can also keep it authenticated and call with an ID token.)

## Local Development

To run this function locally for development and testing, follow these steps.

### Prerequisites

1.  **Python 3.11** or later.
2.  **Google Cloud SDK:** Install the [gcloud CLI](https://cloud.google.com/sdk/docs/install).
3.  **Authentication:** Log in with Application Default Credentials. This allows the function to access GCS assets.
    ```bash
    gcloud auth application-default login
    ```

### Installation

Install the required Python packages from the `video-to-montage` directory:

```bash
pip install -r requirements.txt
```

### Running the Server

A helper script `run_local.sh` is provided to start the local server.

```bash
bash run_local.sh
```

This will start the server using `functions-framework`, typically on `http://localhost:8080`.

Alternatively, you can run the command directly:
```bash
functions-framework --target=generate_montage_json --source=main.py --debug
```

### Sending a Request

You can send a POST request to the local server using `curl` or any API client. An example request body is provided in `example_request.json`.

**Make sure to replace the placeholder GCS URIs in `example_request.json` with your own.**

**`curl` command:**
```bash
curl -X POST http://localhost:8080 \
  -H "Content-Type: application/json" \
  -d @example_request.json
```

The function will process the assets from GCS and upload the resulting `montage.json` to the specified `output_gcs_uri`.
