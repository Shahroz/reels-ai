import argparse
import json
import logging
import os

from lib import detect_pan_groups_from_video

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')

def main():
    parser = argparse.ArgumentParser(description="Detect panorama groups in a local video file.")
    parser.add_argument("video_path", help="Path to the local video file.")
    parser.add_argument("--output_prefix", help="Local directory or GCS prefix for output.", default="output")
    parser.add_argument("--stitch", action="store_true", help="Enable stitching.")
    args = parser.parse_args()

    if not os.path.exists(args.video_path):
        logging.error(f"Video file not found: {args.video_path}")
        return

    logging.info(f"Processing video: {args.video_path}")

    # These are default settings, can be exposed as args if needed.
    settings = {
        "stitch": args.stitch,
        "sample_every": 1,
        "sharp_thr": 25.0,            # Very low sharpness threshold
        "feature": "ORB",
        "min_group_len": 3,           # Back to 3 frames minimum
        "tau_inliers": 0.25,          # Even lower inlier requirement
        "tau_v": 10.0,                 # Stricter vertical movement (reject vertical pans)
        "tau_u_min": 5.0,            # Stronger horizontal movement requirement
        "tau_u_max": 700.0,           # Allow larger displacement
        "tau_scale": 0.15,            # More tolerant scale variation
        "tau_parallax": 4.0,          # More relaxed parallax
        "jpeg_quality": 92,
        "export_all_in_group": False,
        "stitch_warper": "cylindrical",
        "stitch_warper_scale": 1.0,
        "min_pano_aspect_ratio": 1.3,     # Width/height ratio for valid panoramas
        "max_black_border_percent": 30.0  # Max percentage of black pixels allowed
    }

    # Clean output directory if it exists locally
    if not args.output_prefix.startswith("gs://") and os.path.exists(args.output_prefix):
        import shutil
        shutil.rmtree(args.output_prefix)
        logging.info(f"Cleaned existing output directory: {args.output_prefix}")

    try:
        logging.info("Starting panorama group detection...")
        # Note: GCS uploads will likely fail if gcloud is not authenticated,
        # but the JSON result should still be generated and printed.
        results = detect_pan_groups_from_video(args.video_path, args.output_prefix, settings)
        logging.info("Detection finished.")
        
        print("\n--- RESULTS ---")
        print(json.dumps(results, indent=2))
        print("---------------")

    except Exception as e:
        logging.error(f"An error occurred during processing: {e}", exc_info=True)

if __name__ == "__main__":
    main()
