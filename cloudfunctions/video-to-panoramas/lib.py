import os
import tempfile
from typing import Dict, Any, List, Tuple
import logging

import cv2
import numpy as np
from google.cloud import storage

# Set up Cloud Run compatible logging
def setup_logging():
    # Check if running in Cloud Run (has PORT env var)
    if os.getenv('PORT'):
        # Cloud Run environment - use structured logging
        import google.cloud.logging
        client = google.cloud.logging.Client()
        client.setup_logging()
    else:
        # Local environment - use basic logging
        logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')

setup_logging()

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
    if dst_uri.startswith("gs://"):
        bucket_name, blob_path = parse_gcs_uri(dst_uri)
        client = storage.Client()
        bucket = client.bucket(bucket_name)
        blob = bucket.blob(blob_path)
        blob.upload_from_filename(local_path)
        return f"gs://{bucket_name}/{blob_path}"
    else:
        # Local file handling - create directory if needed and copy file
        os.makedirs(os.path.dirname(dst_uri), exist_ok=True)
        import shutil
        shutil.copy2(local_path, dst_uri)
        return dst_uri

def gcs_join(prefix: str, *parts: str) -> str:
    # prefix: gs://bucket/folder or local path
    if prefix.startswith("gs://"):
        bucket, base = parse_gcs_uri(prefix)
        joined = "/".join([p.strip("/") for p in ([base] + list(parts) if base else list(parts))])
        return f"gs://{bucket}/{joined}" if joined else f"gs://{bucket}"
    else:
        # Local path handling
        joined_parts = "/".join([p.strip("/") for p in parts])
        return os.path.join(prefix, joined_parts)

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
    # encode JPEG -> /tmp -> upload or save locally
    fd, tmp_path = tempfile.mkstemp(suffix=".jpg")
    os.close(fd)
    try:
        cv2.imencode(".jpg", img, [int(cv2.IMWRITE_JPEG_QUALITY), int(quality)])[1].tofile(tmp_path)
        return upload_local_file_to_gcs(tmp_path, dst_uri)
    finally:
        if os.path.exists(tmp_path):
            os.remove(tmp_path)

def validate_panorama_quality(pano: np.ndarray, min_width_height_ratio: float = 1.2, max_black_border_percent: float = 15.0) -> bool:
    """Validate panorama quality based on aspect ratio and black borders."""
    height, width = pano.shape[:2]
    
    # Check width/height ratio - for portrait frames, we want wider than tall
    if width / height < min_width_height_ratio:
        return False
    
    # Check for black borders (stitching artifacts)
    gray = cv2.cvtColor(pano, cv2.COLOR_BGR2GRAY) if len(pano.shape) == 3 else pano
    
    # Count black/very dark pixels (threshold < 10)
    black_pixels = np.sum(gray < 10)
    total_pixels = gray.size
    black_percentage = (black_pixels / total_pixels) * 100
    
    return black_percentage <= max_black_border_percent

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
        "sharp_thr": 20.0,            # Laplacian variance threshold
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
    logging.info(f"Starting panorama detection with configuration: {cfg}")

    cap = cv2.VideoCapture(local_video_path)
    if not cap.isOpened():
        raise RuntimeError("Failed to open video")

    total_frames = int(cap.get(cv2.CAP_PROP_FRAME_COUNT))
    fps          = cap.get(cv2.CAP_PROP_FPS) or 0
    width        = int(cap.get(cv2.CAP_PROP_FRAME_WIDTH))
    height       = int(cap.get(cv2.CAP_PROP_FRAME_HEIGHT))
    logging.info(f"Video metadata: total_frames={total_frames}, fps={fps}, width={width}, height={height}")

    sampled = []  # list of (frame_idx, image)
    fidx = 0
    ok, frame = cap.read()
    while ok:
        if fidx % cfg["sample_every"] == 0:
            gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)
            is_frame_sharp = frame_is_sharp(gray, cfg["sharp_thr"])
            if is_frame_sharp:
                sampled.append((fidx, frame.copy()))
        ok, frame = cap.read()
        fidx += 1
    cap.release()
    logging.info(f"Frame sampling complete. Total frames processed: {fidx-1}. Found {len(sampled)} sharp frames to analyze.")

    # Compute motion between consecutive sampled frames
    pair_metrics = []  # between i and i+1 (sampled list indices)
    pan_pairs_count = 0
    for i in range(len(sampled) - 1):
        img1 = sampled[i][1]
        img2 = sampled[i + 1][1]
        m = pair_motion_metrics(img1, img2, cfg["feature"])
        if m is None:
            pair_metrics.append((False, 0.0, None))
            continue
        is_pan, score = is_horizontal_pan(m, cfg)
        if is_pan:
            pan_pairs_count += 1
        pair_metrics.append((is_pan, score, m))
    logging.info(f"Motion analysis complete. Analyzed {len(pair_metrics)} pairs. Found {pan_pairs_count} horizontal pan pairs.")

    # Build groups
    groups = []  # each group is a list of sampled indices
    i = 0
    raw_groups_count = 0
    while i < len(pair_metrics):
        is_pan, score, m = pair_metrics[i]
        if not is_pan:
            i += 1
            continue
        # start group at sampled[i], sampled[i+1]
        raw_groups_count += 1
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
    logging.info(f"Group building complete. Found {raw_groups_count} raw group starts. After filtering by min_group_len={cfg['min_group_len']}, have {len(groups)} groups.")

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
                if status == cv2.Stitcher_OK and validate_panorama_quality(pano, cfg.get("min_pano_aspect_ratio", 1.2), cfg.get("max_black_border_percent", 15.0)):
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
                    pano_uri = None  # stitching failed or quality validation failed

        results["groups"].append(
            {
                "id": gid,
                "direction": "right" if direction == 1 else "left",
                "frame_indices": exported_frame_indices,
                "frame_uris": exported_frame_uris,
                **({"pano_uri": pano_uri} if pano_uri else {}),
            }
        )

    logging.info(f"Finished processing. Returning {len(results['groups'])} groups in final result.")
    return results