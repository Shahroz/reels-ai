import logging
from flask import Request, jsonify
import lib


def generate_montage_json(request: Request):
   """
   HTTP POST JSON:
   {
     "assets": [
       {"type": "video", "gcs_uri": "gs://my-bucket/videos/clip.mp4"},
       {"type": "photo", "gcs_uri": "gs://my-bucket/images/img.jpg"}
     ],
     "output_gcs_uri": "gs://my-bucket/outputs/montage.json",
     "prompt": "Create a fast-paced, energetic promo.",
     "length": 30,
     "resolution": [1920, 1080]
   }
   """
   logging.info("Cloud function generate_montage_json called")
   
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
       assets = payload["assets"]
       output_gcs_uri = payload["output_gcs_uri"]
       prompt = payload.get("prompt")
       length = payload.get("length")
       resolution = payload.get("resolution")
       logging.info(f"Generating montage from {len(assets)} assets to {output_gcs_uri}")
   except KeyError as e:
       logging.error(f"Missing required field: {e}")
       return jsonify(error="Missing required fields: assets, output_gcs_uri"), 400

   try:
       result_gcs_path = lib.process_assets_and_create_montage(
           assets,
           output_gcs_uri,
           prompt=prompt,
           length=length,
           resolution=resolution,
       )
       return jsonify({"output_path": result_gcs_path}), 200
   except Exception as e:
       logging.error(f"Processing failed: {str(e)}", exc_info=True)
       return jsonify(error=str(e)), 500
