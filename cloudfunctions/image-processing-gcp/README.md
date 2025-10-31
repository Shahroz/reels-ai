# Image Processing - Google Cloud Function

This Google Cloud Function processes various image formats (including HEIC, JPEG, WEBP, etc.) and converts them into high-quality PNG files using the `sharp` library.

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
gcloud functions deploy processImage \
  --runtime nodejs20 \
  --trigger-http \
  --allow-unauthenticated \
  --source . \
  --entry-point processImage \
  --memory 2GB \
  --timeout 540s \
  --region us-central1
```

### Environment Variables
To secure your function, set an API token as an environment variable:
```bash
gcloud functions deploy processImage \
  --set-env-vars API_TOKEN=your-secure-api-token
```

## Usage

Send a POST request to the function URL with an authentication header.

### Example with `X-API-Token`
```bash
curl -X POST https://REGION-PROJECT_ID.cloudfunctions.net/processImage \
  -H "Content-Type: application/json" \
  -H "X-API-Token: your-secure-api-token" \
  -d '{
    "imageGcsUri": "gs://your-bucket/path/to/image.heic",
    "outputGcsPath": "gs://your-output-bucket/images/converted-image.png"
  }'
```

### Example with `Authorization` Header
```bash
curl -X POST https://REGION-PROJECT_ID.cloudfunctions.net/processImage \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-secure-api-token" \
  -d '{
    "imageGcsUri": "gs://your-bucket/path/to/image.jpg",
    "outputGcsPath": "gs://your-output-bucket/images/another-image.png"
  }'
```

### Request Body Parameters

- `imageGcsUri`: GCS URI of the input image file (e.g., `gs://bucket/image.jpg`).
- `outputGcsPath`: GCS URI of the output PNG image file (e.g., `gs://bucket/output/converted.png`). The function will convert the input image to PNG format and save it to this path.

## Response

The function returns a JSON response with the GCS URI of the processed image.

### Success Response
```json
{
  "message": "Successfully processed and uploaded image.",
  "imagePath": "gs://your-output-bucket/images/converted-image.png"
}
```

### Error Response
```json
{
  "error": "An unexpected error occurred.",
  "details": "Error message here..."
}
```

## Requirements

- Google Cloud Project with Cloud Functions API enabled.
- Cloud Storage buckets for input images and output PNGs.
- Appropriate IAM permissions for the function's service account to access the buckets.
- Note: For formats like HEIC/HEIF, the underlying Google Cloud Function environment must have `libvips` compiled with `libheif` support. The standard Node.js 20 environment typically includes this.
