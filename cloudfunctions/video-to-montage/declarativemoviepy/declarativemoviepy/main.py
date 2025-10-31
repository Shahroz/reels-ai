import json
import logging
import os
import shutil
import tempfile

from .gcs_utils import AssetManager, is_gcs_path, upload_local_file_to_gcs
from .interpreter import build_clip_from_json

# Setup logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(levelname)s - %(message)s"
)


def render_from_json(composition: dict):
    """
    Renders a video from a JSON composition dictionary.

    This function orchestrates the entire rendering process:
    1. Uses the provided composition dictionary.
    2. Manages GCS assets, downloading them locally for rendering.
    3. Interprets the JSON configuration to build a moviepy Clip.
    4. Renders the clip to a video file.
    5. Uploads the final video to GCS if required.
    6. Cleans up all temporary local files.

    Args:
        composition: A dictionary containing the JSON composition data.
    """
    asset_manager = None
    temp_render_dir = None
    try:
        logging.info("Starting video rendering from composition dictionary")

        # 2. Instantiate AssetManager to handle GCS assets
        asset_manager = AssetManager(composition)

        # 3. Build the final moviepy clip from the JSON config
        logging.info("Building clip from JSON configuration...")
        final_clip = build_clip_from_json(
            composition["clip"], asset_manager.local_asset_map
        )
        
        # 3.5. Add background audio if specified
        if "audio" in composition:
            logging.info("Adding background audio...")
            
            # Build audio clip WITHOUT effects first
            audio_config = composition["audio"].copy()
            audio_effects = audio_config.pop("effects", [])
            
            # Create base audio clip
            audio_clip = build_clip_from_json(audio_config, asset_manager.local_asset_map)
            
            # Adjust duration to match video BEFORE applying effects
            target_duration = final_clip.duration
            if target_duration is not None and audio_clip.duration is not None:
                if audio_clip.duration > target_duration:
                    audio_clip = audio_clip.subclipped(0, target_duration)
                elif audio_clip.duration < target_duration:
                    from moviepy.audio.fx.AudioLoop import AudioLoop
                    audio_clip = audio_clip.with_effects([AudioLoop(duration=target_duration)])
            else:
                logging.warning(f"Cannot adjust audio duration - target_duration: {target_duration}, audio_duration: {audio_clip.duration}")
            
            # NOW apply the effects to the properly sized audio
            if audio_effects:
                from .effect_handlers import apply_effects
                audio_clip = apply_effects(audio_clip, audio_effects)
            
            final_clip = final_clip.with_audio(audio_clip)
            logging.info("Background audio added successfully.")
        
        logging.info("Clip built successfully.")

        # 4. Determine output path and prepare for rendering
        output_path = composition["output_path"]
        local_render_path = output_path
        is_gcs_output = is_gcs_path(output_path)

        if is_gcs_output:
            # Create a temporary local file for rendering
            temp_render_dir = tempfile.mkdtemp(prefix="declarativemoviepy_render_")
            filename = os.path.basename(output_path)
            local_render_path = os.path.join(temp_render_dir, filename)
            logging.info(
                f"GCS output detected. Rendering to temporary path: {local_render_path}"
            )

        # 5. Render the video using clip.write_videofile()
        render_params = {
            "fps": composition.get("fps", 24),  # Default to 24 fps
            "codec": composition.get("codec"),
            "bitrate": composition.get("bitrate"),
            "threads": composition.get("threads"),
            "preset": composition.get("preset", "medium"),
            "audio_codec": composition.get("audio_codec", "aac"),  # Default to AAC audio codec
            "audio": True,  # Ensure audio is included
        }
        # Filter out None values
        render_params = {k: v for k, v in render_params.items() if v is not None}

        logging.info(f"Rendering video to {local_render_path}...")
        final_clip.write_videofile(local_render_path, **render_params)
        logging.info("Video rendering complete.")

        # 6. Upload to GCS if necessary
        if is_gcs_output:
            logging.info(f"Uploading rendered file to {output_path}...")
            upload_local_file_to_gcs(local_render_path, output_path)
            logging.info("Upload to GCS complete.")

    finally:
        # 7. Cleanup temporary files
        if asset_manager:
            asset_manager.cleanup()
        if temp_render_dir and os.path.exists(temp_render_dir):
            shutil.rmtree(temp_render_dir)
            logging.info(f"Cleaned up temporary render directory: {temp_render_dir}")


def render_from_json_file(json_path: str):
    """
    Renders a video from a JSON composition file.

    This function loads the composition from a JSON file and calls render_from_json
    to handle the actual rendering process.

    Args:
        json_path: The path to the JSON composition file.
    """
    try:
        # Load the JSON composition from file
        with open(json_path, "r") as f:
            composition = json.load(f)
        logging.info(f"Successfully loaded composition from {json_path}")
        
        # Call the main rendering function
        render_from_json(composition)
        
    except Exception as e:
        logging.error(f"Failed to render from JSON file {json_path}: {e}")
        raise


if __name__ == "__main__":
    # Example usage:
    # python -m declarativemoviepy.main path/to/your/composition.json
    import sys

    if len(sys.argv) > 1:
        json_file_path = sys.argv[1]
        render_from_json_file(json_file_path)
    else:
        print("Usage: python -m declarativemoviepy.main <path_to_json_file>")