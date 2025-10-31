import os
import logging

from flask import Request, jsonify

import lib


# ----------------------
# Cloud Function (HTTP)
# ----------------------
def extract_pans(request: Request):
    """
    HTTP POST JSON:
    {
      "video_gcs_uri": "gs://my-bucket/videos/clip.mp4",
      "output_prefix": "gs://my-bucket/outputs/run123/",
      "settings": {
        "stitch": true,
        "sample_every": 3,
        "sharp_thr": 20.0,
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
    logging.info("Cloud function extract_pans called")
    
    if request.method != "POST":
        logging.warning(f"Invalid method {request.method}, expected POST")
        return jsonify(error="Use POST with JSON."), 405

    try:
        payload = request.get_json(force=True, silent=False)
        logging.info(f"Received payload with keys: {list(payload.keys()) if payload else 'None'}")
    except Exception as e:
        logging.error(f"Failed to parse JSON: {e}")
        return jsonify(error=f"Invalid JSON: {e}"), 400

    try:
        video_gcs_uri = payload["video_gcs_uri"]
        output_prefix = payload["output_prefix"]
        logging.info(f"Processing video: {video_gcs_uri} -> {output_prefix}")
    except KeyError as e:
        logging.error(f"Missing required field: {e}")
        return jsonify(error="Missing required fields: video_gcs_uri, output_prefix"), 400

    settings = payload.get("settings", {})

    try:
        logging.info("Downloading video from GCS")
        local_path = lib.download_gcs_to_temp(video_gcs_uri)
        try:
            logging.info("Starting panorama detection")
            results = lib.detect_pan_groups_from_video(local_path, output_prefix, settings)
            logging.info(f"Processing completed successfully, found {len(results.get('groups', []))} groups")
        finally:
            if os.path.exists(local_path):
                os.remove(local_path)
                logging.info("Cleaned up temporary video file")
        return jsonify(results), 200
    except Exception as e:
        logging.error(f"Processing failed: {str(e)}", exc_info=True)
        return jsonify(error=str(e)), 500
