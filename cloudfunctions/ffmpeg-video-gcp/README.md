# FFMPEG Video Frame Extraction - Google Cloud Function

This Google Cloud Function extracts frames from video files at specified timestamps using FFMPEG.

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

### Deploy the Function

```bash
# Install dependencies
npm install

# Deploy using npm script
npm run deploy
```

Or deploy manually with more options:
```bash
gcloud functions deploy extractVideoFrames \
  --runtime nodejs20 \
  --trigger-http \
  --allow-unauthenticated \
  --source . \
  --entry-point extractVideoFrames \
  --memory 2GB \
  --timeout 540s \
  --region us-central1
```

### Environment Variables
```bash
gcloud functions deploy extractVideoFrames \
  --set-env-vars API_TOKEN=your-secure-api-token
```

## Usage

Send a POST request to the function URL with authentication header:

```bash
curl -X POST https://REGION-PROJECT_ID.cloudfunctions.net/extractVideoFrames \
  -H "Content-Type: application/json" \
  -H "X-API-Token: your-secure-api-token" \
  -d '{
    "videoGcsUri": "gs://your-bucket/path/to/video.mp4",
    "timestamps": [1000, 5000, 10000],
    "outputGcsPath": "gs://your-output-bucket/frames/"
  }'
```

Or using Authorization header:
```bash
curl -X POST https://REGION-PROJECT_ID.cloudfunctions.net/extractVideoFrames \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-secure-api-token" \
  -d '{
    "videoGcsUri": "gs://your-bucket/path/to/video.mp4",
    "timestamps": [1000, 5000, 10000],
    "outputGcsPath": "gs://your-output-bucket/frames/"
  }'
```

### Request Body Parameters

- `videoGcsUri`: GCS URI of the input video file
- `timestamps`: Array of timestamps in milliseconds where frames should be extracted
- `outputGcsPath`: GCS path where extracted frames will be saved

## Response

The function returns a JSON response with:

```json
{
  "message": "Successfully extracted and uploaded N frames.",
  "framePaths": [
    "gs://bucket-name/path/frame_1000.png",
    "gs://bucket-name/path/frame_5000.png",
    "gs://bucket-name/path/frame_10000.png"
  ]
}
```

- `message`: Success message with frame count
- `framePaths`: Array of GCS URIs (gs://) for all extracted frame images

## Requirements

- Google Cloud Project with Cloud Functions API enabled
- Cloud Storage buckets for input video and output frames
- Appropriate IAM permissions for the function's service account to access the buckets