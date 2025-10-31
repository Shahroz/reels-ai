# Photo Extraction Services

This module provides services for processing photos and video frames, including:

## Video Frame Extraction

The `extract_frames_from_video_on_gcs` service extracts specific frames from video files stored in Google Cloud Storage using ffmpeg.

### Dependencies
- `ffmpeg` must be installed and available in PATH

### Usage
```rust
use crate::services::photo_extraction::extract_frames_from_video_on_gcs::extract_frames_from_video_on_gcs;

let gcs_uri = "gs://my-bucket/video.mp4";
let timestamps = ["00:00:02", "00:00:05", "00:00:08"];
let frame_paths = extract_frames_from_video_on_gcs(gcs_uri, &timestamps).await?;
```

## RAW Image Conversion (HEIC & DNG)

The `convert_heic_to_png` service automatically converts RAW image files to web-compatible formats for browser compatibility. This conversion is integrated into the asset upload flow and supports:

- **HEIC** (High Efficiency Image Container) - Apple's modern image format
- **DNG** (Digital Negative) - Adobe's open RAW format

### Why Convert RAW Images?

RAW formats like HEIC and DNG provide excellent image quality and metadata preservation, but they're not widely supported by web browsers. Converting to web-compatible formats (WebP by default, PNG optional) ensures:

- ✅ Universal browser compatibility
- ✅ Consistent display across all platforms  
- ✅ Better integration with web-based tools
- ✅ No client-side compatibility issues
- ✅ Optimized file sizes for web delivery

### Dependencies

ImageMagick with appropriate format support must be installed:

#### macOS (using Homebrew)
```bash
brew install imagemagick libheif  # libheif for HEIC, libraw usually included
```

#### Ubuntu/Debian
```bash
apt-get update
apt-get install imagemagick libheif-examples libheif-dev libraw-bin
```

#### Alpine Linux
```bash
apk add imagemagick libheif libheif-dev libraw
```

#### CentOS/RHEL/Fedora
```bash
# Enable EPEL repository for CentOS/RHEL
yum install epel-release

# Install packages
yum install ImageMagick libheif libheif-devel libraw
```

### Verification

To verify ImageMagick has HEIC support:
```bash
magick identify -list format | grep -i heic
```

You should see output like:
```
    HEIC* HEIC      rw+   High Efficiency Image Container
```

To verify ImageMagick has DNG support:
```bash
magick identify -list format | grep -i dng
```

You should see output like:
```
    DNG  DNG       r--   Digital Negative Raw Format (0.21.4-Release)
```

### How It Works

The RAW image conversion (HEIC/DNG) is automatically triggered during the asset upload confirmation process:

1. **Upload**: User uploads a RAW file (HEIC/DNG) using the standard upload flow
2. **Detection**: The `confirm_upload` endpoint detects the content type:
   - `image/heic` for HEIC files
   - `image/x-adobe-dng` for DNG files
3. **Conversion**: ImageMagick converts the RAW file to web-compatible format (WebP by default)
4. **Storage**: The converted file is stored in GCS
5. **Cleanup**: The original RAW file is deleted from GCS
6. **Database**: The asset record is created with the converted file details

### Supported Formats

| Input Format | Content Type | Output Formats | Compression |
|--------------|--------------|----------------|-------------|
| HEIC | `image/heic` | WebP (default), PNG | Excellent compression |
| DNG | `image/x-adobe-dng` | WebP (default), PNG | ~14% of original size (WebP) |

**Test Results** (with Leica DNG file):
- Original DNG: 17.49 MB
- WebP output: 2.47 MB (14.1% compression)
- PNG output: 90.70 MB (518.7% - larger due to no compression)

### Integration

The conversion is integrated into `routes/assets/confirm_upload.rs`:

```rust
// Detect RAW files and convert to web-compatible format
let (final_object_name, final_content_type, final_file_name) = if content_type == "image/heic" {
    // HEIC conversion
    match convert_heic_on_gcs(&gcs_client, &bucket_name, &object_name, None).await {
        // ... conversion logic
    }
} else if content_type == "image/x-adobe-dng" {
    // DNG conversion  
    match convert_dng_on_gcs(&gcs_client, &bucket_name, &object_name, None).await {
        // ... conversion logic
    }
} else {
    (object_name, content_type, file_name)
};
```

### Usage

No special action required from users - RAW files are automatically converted:

```typescript
// Frontend - upload RAW files normally
const file = document.querySelector('input[type="file"]').files[0]; // my-photo.heic or image.dng
await uploadAsset(file); // Automatically converts to WebP on backend
```

### Performance Considerations

- **Temporary Storage**: Uses `/tmp` directory for processing, automatically cleaned up
- **Memory Usage**: Files are processed efficiently using ImageMagick's optimized processing
- **Processing Time**: 
  - HEIC conversion: typically 1-3 seconds for mobile photos
  - DNG conversion: typically 2-5 seconds for RAW files (depends on file size)
- **File Size**: 
  - WebP output: ~14% of original DNG size (excellent compression)
  - PNG output: Can be larger than original for some formats, but provides universal compatibility

### Error Handling

The service provides detailed error messages for common issues:

- Missing ImageMagick installation
- Missing libheif support (for HEIC)
- Missing libraw support (for DNG)
- Corrupted RAW files
- GCS upload/download failures
- Insufficient disk space in `/tmp`

### Monitoring

Conversion events are logged for monitoring:

```
INFO Successfully converted HEIC to WEBP: user123/abc-123.heic -> user123/abc-123.webp (0.8MB)
INFO Successfully converted DNG to WEBP: user123/L1004220.dng -> user123/L1004220.webp (2.5MB)
```

### Testing

Run the integration tests (requires real GCS access and ImageMagick):

```bash
cd crates/narrativ/backend
cargo test test_convert_heic_to_png --ignored
``` 