Awesome—here’s a plug-and-play **Google Cloud Function (HTTP)** in Python that:

* takes a **GCS video URI** and an **output GCS folder/prefix**,
* detects horizontally panning sequences,
* optionally **stitches panoramas**,
* uploads results (selected frames and/or panoramas) to your output prefix,
* and **returns JSON** listing GCS paths for each group.

---

# Deploy (2nd gen)

```bash
gcloud functions deploy extract_pans \
  --gen2 \
  --runtime=python311 \
  --region=us-central1 \
  --entry-point=extract_pans \
  --source=. \
  --trigger-http \
  --allow-unauthenticated
```

(Adjust region, auth policy, and project as needed. You can also keep it authenticated and call with an ID token.)

---

# Example request

```bash
curl -X POST "$FUNCTION_URL" \
  -H "Content-Type: application/json" \
  -d '{
    "video_gcs_uri": "gs://my-bucket/input/video1.mp4",
    "output_gcs_prefix": "gs://my-bucket/outputs/run_001/",
    "settings": {
      "stitch": true,
      "sample_every": 3,
      "sharp_thr": 80.0,
      "feature": "ORB",
      "min_group_len": 3,
      "tau_inliers": 0.3,
      "tau_v": 1.5,
      "tau_u_min": 15.0,
      "tau_u_max": 400.0,
      "tau_scale": 0.02,
      "tau_parallax": 1.5,
      "jpeg_quality": 92,
      "export_all_in_group": false,
      "stitch_warper": "cylindrical",
      "stitch_warper_scale": 1.0
    }
  }'
```

# Example response

```json
{
  "video_meta": {
    "total_frames": 12345,
    "fps": 29.97,
    "width": 1920,
    "height": 1080,
    "sample_every": 3
  },
  "groups": [
    {
      "id": 1,
      "direction": "right",
      "frame_indices": [120, 129, 141, 162],
      "frame_uris": [
        "gs://my-bucket/outputs/run_001/groups/group001/frame_0000120.jpg",
        "gs://my-bucket/outputs/run_001/groups/group001/frame_0000129.jpg",
        "gs://my-bucket/outputs/run_001/groups/group001/frame_0000141.jpg",
        "gs://my-bucket/outputs/run_001/groups/group001/frame_0000162.jpg"
      ],
      "pano_uri": "gs://my-bucket/outputs/run_001/panos/group001.jpg"
    },
    {
      "id": 2,
      "direction": "left",
      "frame_indices": [345, 354, 366, 381],
      "frame_uris": [
        "gs://my-bucket/outputs/run_001/groups/group002/frame_0000345.jpg",
        "gs://my-bucket/outputs/run_001/groups/group002/frame_0000354.jpg",
        "gs://my-bucket/outputs/run_001/groups/group002/frame_0000366.jpg",
        "gs://my-bucket/outputs/run_001/groups/group002/frame_0000381.jpg"
      ]
    }
  ]
}
```

---

## Notes & knobs

* **Settings** are optional; sensible defaults are used. If your footage is 4K or very shaky, you may want to increase `sharp_thr` a bit and relax `tau_v` (e.g., 2–3).
* To export **every** sampled frame in a group, set `"export_all_in_group": true`.
* If panoramas fail for some groups, those entries simply omit `pano_uri` (OpenCV returns non-OK status).
* Function stores temporary files under `/tmp` (Cloud Functions’ writable temp dir).
* Make sure the service account running the function has **Storage Object Viewer** on the input bucket and **Storage Object Admin** (or Writer) on the output bucket/prefix.

If you want this as a **Cloud Run service** (for larger videos / longer timeouts) or you want **Pub/Sub**-triggered processing, I can hand you that variant too.