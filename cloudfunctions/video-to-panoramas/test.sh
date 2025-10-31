curl -X POST "https://us-central1-bounti-prod-322900.cloudfunctions.net/extract_pans" \
  -H "Content-Type: application/json" \
  -d '{
    "video_gcs_uri": "gs://real-estate-videos/test_videos/7b4435e5-d92c-40e1-b731-e7cae3fd1c50.mov",
    "output_prefix": "gs://real-estate-videos/test_videos/extract_pans_test/run_002/",
    "settings": {
        "stitch": true,
        "sample_every": 1,
        "sharp_thr": 25.0,
        "feature": "ORB",
        "min_group_len": 3,
        "tau_inliers": 0.25,
        "tau_v": 10.0,
        "tau_u_min": 5.0,
        "tau_u_max": 700.0,
        "tau_scale": 0.15,
        "tau_parallax": 4.0,
        "jpeg_quality": 92,
        "export_all_in_group": false,
        "stitch_warper": "cylindrical",
        "stitch_warper_scale": 1.0,
        "min_pano_aspect_ratio": 1.3,
        "max_black_border_percent": 30.0
    }
  }'