Awesome—here’s a plug-and-play **Google Cloud Function (HTTP)** in Python that:

* takes a **GCS video URI** and an **output GCS folder/prefix**,
* detects horizontally panning sequences,
* optionally **stitches panoramas**,
* uploads results (selected frames and/or panoramas) to your output prefix,
* and **returns JSON** listing GCS paths for each group.

I included deployment instructions, `requirements.txt`, and a sample payload.

---

# Files

## `main.py`

```python
import json
import os
import tempfile
from typing import Dict, Any, List, Tuple

import cv2
import numpy as np
from google.cloud import storage
from flask import Request, jsonify

# ----------------------
# Utility: GCS handling
# ----------------------
def parse_gcs_uri(uri: str) -> Tuple[str, str]:
    # gs://bucket/path/to/object
    if not uri.startswith("gs://"):
        raise ValueError("Expected GCS URI starting with gs://")
    _, rest = uri.split("gs://", 1)
    bucket, *path = rest.split("/", 1)
    blob_path = path[0] if path else ""
    return bucket, blob_path

def download_gcs_to_temp(uri: str) -> str:
    bucket_name, blob_path = parse_gcs_uri(uri)
    client = storage.Client()
    bucket = client.bucket(bucket_name)
    blob = bucket.blob(blob_path)
    if not blob.exists(client):
        raise FileNotFoundError(f"GCS object not found: {uri}")
    fd, local_path = tempfile.mkstemp(suffix=os.path.splitext(blob_path)[1])
    os.close(fd)
    blob.download_to_filename(local_path)
    return local_path

def upload_local_file_to_gcs(local_path: str, dst_uri: str) -> str:
    bucket_name, blob_path = parse_gcs_uri(dst_uri)
    client = storage.Client()
    bucket = client.bucket(bucket_name)
    blob = bucket.blob(blob_path)
    blob.upload_from_filename(local_path)
    return f"gs://{bucket_name}/{blob_path}"

def gcs_join(prefix: str, *parts: str) -> str:
    # prefix: gs://bucket/folder
    if not prefix.startswith("gs://"):
        raise ValueError("gcs_join: prefix must start with gs://")
    bucket, base = parse_gcs_uri(prefix)
    joined = "/".join([p.strip("/") for p in ([base] + list(parts) if base else list(parts))])
    return f"gs://{bucket}/{joined}" if joined else f"gs://{bucket}"

# ----------------------
# Vision utils
# ----------------------
def frame_is_sharp(gray: np.ndarray, thr: float) -> bool:
    # Laplacian variance
    return float(cv2.Laplacian(gray, cv2.CV_64F).var()) > thr

def pair_motion_metrics(img1: np.ndarray, img2: np.ndarray, feat: str = "ORB") -> Dict[str, Any] | None:
    gray1 = cv2.cvtColor(img1, cv2.COLOR_BGR2GRAY)
    gray2 = cv2.cvtColor(img2, cv2.COLOR_BGR2GRAY)

    if feat.upper() == "SIFT":
        sift = cv2.SIFT_create(nfeatures=4000)
        k1, d1 = sift.detectAndCompute(gray1, None)
        k2, d2 = sift.detectAndCompute(gray2, None)
        norm = cv2.NORM_L2
    else:
        orb = cv2.ORB_create(4000)
        k1, d1 = orb.detectAndCompute(gray1, None)
        k2, d2 = orb.detectAndCompute(gray2, None)
        norm = cv2.NORM_HAMMING

    if d1 is None or d2 is None or len(k1) == 0 or len(k2) == 0:
        return None

    bf = cv2.BFMatcher(norm, crossCheck=False)
    raw = bf.knnMatch(d1, d2, k=2)

    good = [m for m, n in raw if n is not None and m.distance < 0.75 * n.distance]
    if len(good) < 30:
        return None

    pts1 = np.float32([k1[m.queryIdx].pt for m in good])
    pts2 = np.float32([k2[m.trainIdx].pt for m in good])

    H, inliers = cv2.findHomography(pts1, pts2, cv2.RANSAC, 3.0)
    if H is None or inliers is None:
        return None

    inmask = inliers.ravel().astype(bool)
    if inmask.sum() < 20:
        return None

    P1, P2 = pts1[inmask], pts2[inmask]
    flows = P2 - P1
    median_u = float(np.median(flows[:, 0]))
    median_v = float(np.median(flows[:, 1]))
    inlier_ratio = float(inmask.mean())

    # residuals after applying H
    P1h = cv2.perspectiveTransform(P1.reshape(-1, 1, 2), H).reshape(-1, 2)
    resid = float(np.linalg.norm(P2 - P1h, axis=1).mean())

    # crude scale from H (2x2 submatrix)
    s = float(
        np.sqrt(
            (H[0, 0] ** 2 + H[1, 0] ** 2 + H[0, 1] ** 2 + H[1, 1] ** 2) / 2.0
        )
    )

    return dict(H=H, inlier_ratio=inlier_ratio, median_u=median_u,
                median_v=median_v, resid=resid, scale=s)

def is_horizontal_pan(m: Dict[str, Any], cfg: Dict[str, Any]) -> Tuple[bool, float]:
    if m is None:
        return False, 0.0
    ok = (
        m["inlier_ratio"] >= cfg["tau_inliers"]
        and abs(m["median_v"]) <= cfg["tau_v"]
        and cfg["tau_u_min"] <= abs(m["median_u"]) <= cfg["tau_u_max"]
        and abs(m["scale"] - 1.0) <= cfg["tau_scale"]
        and m["resid"] <= cfg["tau_parallax"]
    )
    score = (
        1.5 * m["inlier_ratio"]
        + 0.6 * (1 - min(abs(m["median_v"]) / cfg["tau_v"], 1))
        + 0.4 * (1 - min(abs(m["scale"] - 1.0) / cfg["tau_scale"], 1))
        + 0.6 * (1 - min(m["resid"] / cfg["tau_parallax"], 1))
    )
    return ok, float(score)

def save_frame_to_gcs(img: np.ndarray, dst_uri: str, quality: int = 92) -> str:
    # encode JPEG -> /tmp -> upload
    fd, tmp_path = tempfile.mkstemp(suffix=".jpg")
    os.close(fd)
    try:
        cv2.imencode(".jpg", img, [int(cv2.IMWRITE_JPEG_QUALITY), int(quality)])[1].tofile(tmp_path)
        return upload_local_file_to_gcs(tmp_path, dst_uri)
    finally:
        if os.path.exists(tmp_path):
            os.remove(tmp_path)

def stitch_group(frames: List[np.ndarray], warper: str = "cylindrical", warper_scale: float = 1.0):
    # Use OpenCV's Stitcher; prefer PANORAMA mode
    try:
        stitcher = cv2.Stitcher_create(cv2.Stitcher_PANORAMA)
    except AttributeError:
        stitcher = cv2.createStitcher(cv2.Stitcher_PANORAMA)
    # Warper selection
    try:
        stitcher.setWarper(cv2.PyRotationWarper(warper, warper_scale))
    except Exception:
        pass  # fallback to defaults if unavailable

    status, pano = stitcher.stitch(frames)
    return status, pano

# ----------------------
# Core: process video
# ----------------------
def detect_pan_groups_from_video(
    local_video_path: str,
    gcs_output_prefix: str,
    settings: Dict[str, Any],
) -> Dict[str, Any]:
    # Defaults (tuned for 1080p; scale with resolution if needed)
    cfg = {
        "sample_every": 3,            # sample every k frames
        "min_group_len": 3,           # at least N frames in a group
        "sharp_thr": 80.0,            # Laplacian variance threshold
        "feature": "ORB",             # ORB or SIFT
        "tau_inliers": 0.30,
        "tau_v": 1.5,
        "tau_u_min": 15.0,
        "tau_u_max": 400.0,
        "tau_scale": 0.02,
        "tau_parallax": 1.5,
        "export_overlap_target": 0.4, # ~40% overlap for export subset
        "jpeg_quality": 92,
        "stitch": False,
        "stitch_warper": "cylindrical",
        "stitch_warper_scale": 1.0,
        "export_all_in_group": False, # if True, export every sampled frame in group
    }
    cfg.update(settings or {})

    cap = cv2.VideoCapture(local_video_path)
    if not cap.isOpened():
        raise RuntimeError("Failed to open video")

    total_frames = int(cap.get(cv2.CAP_PROP_FRAME_COUNT))
    fps          = cap.get(cv2.CAP_PROP_FPS) or 0
    width        = int(cap.get(cv2.CAP_PROP_FRAME_WIDTH))
    height       = int(cap.get(cv2.CAP_PROP_FRAME_HEIGHT))

    sampled = []  # list of (frame_idx, image)
    fidx = 0
    ok, frame = cap.read()
    while ok:
        if fidx % cfg["sample_every"] == 0:
            gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)
            if frame_is_sharp(gray, cfg["sharp_thr"]):
                sampled.append((fidx, frame.copy()))
        ok, frame = cap.read()
        fidx += 1
    cap.release()

    # Compute motion between consecutive sampled frames
    pair_metrics = []  # between i and i+1 (sampled list indices)
    for i in range(len(sampled) - 1):
        img1 = sampled[i][1]
        img2 = sampled[i + 1][1]
        m = pair_motion_metrics(img1, img2, cfg["feature"])
        if m is None:
            pair_metrics.append((False, 0.0, None))
            continue
        is_pan, score = is_horizontal_pan(m, cfg)
        pair_metrics.append((is_pan, score, m))

    # Build groups
    groups = []  # each group is a list of sampled indices
    i = 0
    while i < len(pair_metrics):
        is_pan, score, m = pair_metrics[i]
        if not is_pan:
            i += 1
            continue
        # start group at sampled[i], sampled[i+1]
        direction = 1 if m["median_u"] >= 0 else -1
        g = [i, i + 1]  # indices into sampled
        j = i + 1
        while j < len(pair_metrics):
            jp = pair_metrics[j]
            if not jp[0]:
                break
            m2 = jp[2]
            dir2 = 1 if m2["median_u"] >= 0 else -1
            if dir2 != direction:
                break
            g.append(j + 1)  # extend by next frame
            j += 1
        # Dedup consecutive indices
        g = sorted(set(g))
        if len(g) >= cfg["min_group_len"]:
            groups.append((direction, g))
        i = j + 1

    # Export frames & (optional) stitch
    results = {
        "video_meta": {
            "total_frames": total_frames,
            "fps": fps,
            "width": width,
            "height": height,
            "sample_every": cfg["sample_every"],
        },
        "groups": []
    }

    # Helper: pick export subset with ~target overlap by stepping indices
    def pick_subset(indices: List[int], target_step: int) -> List[int]:
        out = []
        last = None
        for idx in indices:
            if last is None or idx - last >= target_step:
                out.append(idx)
                last = idx
        if out[-1] != indices[-1]:
            out.append(indices[-1])
        return out

    # Estimate a reasonable step in sampled units from tau_u bounds
    # Conservative default: keep every frame; caller can toggle export_all_in_group
    for gid, (direction, sampled_idxs) in enumerate(groups, start=1):
        frames_info = []
        # Decide which exact sampled frames to export
        if cfg["export_all_in_group"]:
            export_idxs = sampled_idxs
        else:
            # heuristic: keep ~every 2nd-3rd sampled frame
            step = max(1, int(round(2)))
            export_idxs = pick_subset(sampled_idxs, step)

        # Upload selected frames
        exported_frame_uris = []
        exported_frame_indices = []
        for sidx in export_idxs:
            frame_idx, img = sampled[sidx]
            out_uri = gcs_join(
                gcs_output_prefix,
                "groups",
                f"group{gid:03d}",
                f"frame_{frame_idx:07d}.jpg",
            )
            uploaded = save_frame_to_gcs(img, out_uri, cfg["jpeg_quality"])
            exported_frame_uris.append(uploaded)
            exported_frame_indices.append(frame_idx)

        pano_uri = None
        if cfg["stitch"]:
            # gather actual images again (ensure consistent order)
            frames_for_stitch = [sampled[s][1] for s in export_idxs]
            if len(frames_for_stitch) >= 2:
                status, pano = stitch_group(
                    frames_for_stitch,
                    cfg["stitch_warper"],
                    float(cfg["stitch_warper_scale"]),
                )
                if status == cv2.Stitcher_OK:
                    fd, tmp_pano = tempfile.mkstemp(suffix=".jpg")
                    os.close(fd)
                    try:
                        cv2.imwrite(tmp_pano, pano)
                        pano_uri = gcs_join(gcs_output_prefix, "panos", f"group{gid:03d}.jpg")
                        pano_uri = upload_local_file_to_gcs(tmp_pano, pano_uri)
                    finally:
                        if os.path.exists(tmp_pano):
                            os.remove(tmp_pano)
                else:
                    pano_uri = None  # stitching failed; omit

        results["groups"].append(
            {
                "id": gid,
                "direction": "right" if direction == 1 else "left",
                "frame_indices": exported_frame_indices,
                "frame_uris": exported_frame_uris,
                **({"pano_uri": pano_uri} if pano_uri else {}),
            }
        )

    return results

# ----------------------
# Cloud Function (HTTP)
# ----------------------
def extract_pans(request: Request):
    """
    HTTP POST JSON:
    {
      "video_gcs_uri": "gs://my-bucket/videos/clip.mp4",
      "output_gcs_prefix": "gs://my-bucket/outputs/run123/",
      "settings": {
        "stitch": true,
        "sample_every": 3,
        "sharp_thr": 80.0,
        "feature": "ORB",            // or "SIFT"
        "min_group_len": 3,
        "tau_inliers": 0.30,
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
    }
    """
    if request.method != "POST":
        return jsonify(error="Use POST with JSON."), 405

    try:
        payload = request.get_json(force=True, silent=False)
    except Exception as e:
        return jsonify(error=f"Invalid JSON: {e}"), 400

    try:
        video_gcs_uri = payload["video_gcs_uri"]
        output_gcs_prefix = payload["output_gcs_prefix"]
    except KeyError:
        return jsonify(error="Missing required fields: video_gcs_uri, output_gcs_prefix"), 400

    settings = payload.get("settings", {})

    try:
        local_path = download_gcs_to_temp(video_gcs_uri)
        try:
            results = detect_pan_groups_from_video(local_path, output_gcs_prefix, settings)
        finally:
            if os.path.exists(local_path):
                os.remove(local_path)
        return jsonify(results), 200
    except Exception as e:
        # You may log more details with stack traces if desired
        return jsonify(error=str(e)), 500
```

---

## `requirements.txt`

```
google-cloud-storage==2.18.2
opencv-python-headless==4.9.0.80
numpy==1.26.4
Flask==3.0.3
```

> If you deploy on Cloud Functions 2nd gen with Python 3.11, these versions are safe. If you need CUDA or non-headless OpenCV, switch runtimes (but headless is recommended for CF).

---

# Deploy (2nd gen)

```bash
gcloud functions deploy extract_pans \
  --gen2 \
  --runtime=python311 \
  --region=europe-central2 \
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
