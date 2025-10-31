import logging
import json
import time
import tempfile
import os
from google.cloud import storage
import re
from google import genai
from google.genai import types
from pathlib import Path

from declarativemoviepy.declarativemoviepy.validator import validate_composition_json, SchemaValidationError
from declarativemoviepy.declarativemoviepy.main import render_from_json
from declarativemoviepy.declarativemoviepy.gcs_utils import download_gcs_file, get_video_metadata, get_image_metadata

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

def _load_json_schema() -> str:
    """Load the JSON schema from the declarativemoviepy directory."""
    schema_path = Path(__file__).parent / "declarativemoviepy" / "composition_schema.json"
    try:
        with open(schema_path, 'r', encoding='utf-8') as f:
            schema = json.load(f)
            return json.dumps(schema, indent=2)
    except FileNotFoundError:
        logging.warning(f"Schema file not found at {schema_path}, continuing without schema in prompt")
        return ""
    except Exception as e:
        logging.warning(f"Failed to load schema: {e}, continuing without schema in prompt")
        return ""

def _load_example_compositions() -> str:
    """Load example compositions from the declarativemoviepy examples directory."""
    examples_dir = Path(__file__).parent / "declarativemoviepy" / "examples"
    examples = []
    
    try:
        if examples_dir.exists():
            # Load a few representative examples
            example_files = ["simple_composition.json", "real_estate_tour_2.json", "effects.json"]
            for example_file in example_files:
                example_path = examples_dir / example_file
                if example_path.exists():
                    with open(example_path, 'r', encoding='utf-8') as f:
                        example_data = json.load(f)
                        examples.append(f"Example: {example_file}")
                        examples.append(json.dumps(example_data, indent=2))
                        examples.append("")  # Add spacing
    except Exception as e:
        logging.warning(f"Failed to load examples: {e}")
    
    return "\n".join(examples)

def _upload_to_gcs(bucket_name, blob_name, data):
    """Uploads data to a GCS bucket."""
    client = storage.Client()
    bucket = client.bucket(bucket_name)
    blob = bucket.blob(blob_name)
    blob.upload_from_string(data, content_type='application/json')
    logging.info(f"Uploaded to gs://{bucket_name}/{blob_name}")
    return f"gs://{bucket_name}/{blob_name}"

def _timestamp_to_seconds(ts: str) -> float:
    """Converts MM:SS or HH:MM:SS timestamp to seconds."""
    parts = list(map(int, ts.split(':')))
    if len(parts) == 2:
        return float(parts[0] * 60 + parts[1])
    elif len(parts) == 3:
        return float(parts[0] * 3600 + parts[1] * 60 + parts[2])
    return 0.0

def _analyze_video(gcs_uri: str) -> dict:
    """Analyzes a video using Gemini."""
    logging.info(f"Analyzing video: {gcs_uri}")
    client = genai.Client(vertexai=True, project="bounti-prod-322900", location='us-central1')

    prompt = """
Analyze the provided video and generate a JSON object with three keys: "video_analysis", "promo_segments", and "photo_opportunities".

1.  "video_analysis": A list of objects, each with a "timestamp" (MM:SS) and a "description" of the visual content at that time. If speech is present, include a "speech" key.
2.  "promo_segments": A list of objects representing compelling segments for a promotional video. Each object should have "start_timestamp" (MM:SS), "end_timestamp" (MM:SS), and a "description" of why it's a good segment.
3.  "photo_opportunities": A list of objects identifying timestamps with good potential for still photos. Each object should have a "timestamp" (MM:SS), a "description" of the frame, and a list of "arrangement_options". Each "arrangement_option" should have a "title" and a "prompt" for an AI image generator to enhance or virtually stage the photo.

The output MUST be a valid JSON object. Do not include any text before or after the JSON object.
"""

    video_part = types.Part(
        file_data=types.FileData(mime_type="video/mp4", file_uri=gcs_uri)
    )
    
    max_retries = 3
    for attempt in range(max_retries):
        try:
            response = client.models.generate_content(
                model="gemini-2.5-pro",
                contents=[prompt, video_part],
            )
           
            text_response = response.text
            # Robust JSON parsing
            match = re.search(r'```json\s*([\s\S]*?)\s*```', text_response)
            if match:
                json_str = match.group(1)
            else:
                start_index = text_response.find('{')
                end_index = text_response.rfind('}')
                if start_index != -1 and end_index != -1 and start_index < end_index:
                    json_str = text_response[start_index:end_index+1]
                else:
                    json_str = text_response
           
            analysis_result = json.loads(json_str)
            analysis_result['source'] = gcs_uri
            analysis_result['analysis_type'] = 'video'
            return analysis_result
        except Exception as e:
            logging.warning(f"Attempt {attempt + 1} to analyze video {gcs_uri} failed: {e}")
            if attempt + 1 == max_retries:
                logging.error(f"Failed to analyze video {gcs_uri} after {max_retries} attempts.")
                return {"analysis_type": "video", "source": gcs_uri, "promo_segments": [], "photo_opportunities": [], "error": str(e)}
            time.sleep(2 ** attempt)
    return {"analysis_type": "video", "source": gcs_uri, "promo_segments": [], "photo_opportunities": [], "error": "Max retries exceeded"}

def _extract_asset_metadata(assets: list) -> dict:
    """
    Downloads assets and extracts metadata (duration, resolution) for each asset.
    Returns a dictionary mapping GCS URIs to their metadata.
    """
    asset_metadata = {}
    temp_dir = tempfile.mkdtemp(prefix="asset_metadata_")
    
    try:
        for asset in assets:
            gcs_uri = asset.get("gcs_uri")
            asset_type = asset.get("type")
            
            if not gcs_uri or not asset_type:
                continue
                
            try:
                local_path = download_gcs_file(gcs_uri, temp_dir)
                
                if asset_type == "video":
                    metadata = get_video_metadata(local_path)
                    asset_metadata[gcs_uri] = {
                        "type": "video",
                        "duration": metadata.get("duration", 0),
                        "width": metadata.get("width", 0),
                        "height": metadata.get("height", 0),
                        "fps": metadata.get("fps", 24),
                    }
                elif asset_type == "photo":
                    metadata = get_image_metadata(local_path)
                    asset_metadata[gcs_uri] = {
                        "type": "image",
                        "width": metadata.get("width", 0),
                        "height": metadata.get("height", 0),
                    }
                    
                logging.info(f"Extracted metadata for {gcs_uri}: {asset_metadata[gcs_uri]}")
                
            except Exception as e:
                logging.warning(f"Failed to extract metadata for {gcs_uri}: {e}")
                asset_metadata[gcs_uri] = {"type": asset_type, "error": str(e)}
                
    finally:
        # Clean up temporary directory
        import shutil
        if os.path.exists(temp_dir):
            shutil.rmtree(temp_dir)
            logging.info(f"Cleaned up metadata extraction temp directory: {temp_dir}")
    
    return asset_metadata


def _analyze_photo(gcs_uri: str) -> dict:
    """Analyzes a photo using Gemini."""
    logging.info(f"Analyzing photo: {gcs_uri}")
    client = genai.Client(vertexai=True, project="bounti-prod-322900", location='global')
    prompt = "Provide a concise, one-sentence description of this image."
    
    image_part = types.Part(
        file_data=types.FileData(mime_type="image/png", file_uri=gcs_uri)
    )
    
    max_retries = 3
    for attempt in range(max_retries):
        try:
            response = client.models.generate_content(
                model="gemini-2.5-flash",
                contents=[prompt, image_part]
            )
            return {
                "analysis_type": "photo", 
                "source": gcs_uri, 
                "description": response.text.strip()
            }
        except Exception as e:
            logging.warning(f"Attempt {attempt + 1} to analyze photo {gcs_uri} failed: {e}")
            if attempt + 1 == max_retries:
                logging.error(f"Failed to analyze photo {gcs_uri} after {max_retries} attempts.")
                return {"analysis_type": "photo", "source": gcs_uri, "description": "Analysis failed.", "error": str(e)}
            time.sleep(2 ** attempt)
    return {"analysis_type": "photo", "source": gcs_uri, "description": "Analysis failed.", "error": "Max retries exceeded"}

def _fix_montage_json(
   current_json: dict,
   error_message: str,
   analyses: list,
   asset_metadata: dict = None,
   prompt: str = None,
   length: int = None,
   resolution: list = None,
) -> dict:
   """Fixes a montage JSON based on rendering error feedback using Gemini."""
   logging.info("Fixing montage JSON based on error feedback via LLM")

   client = genai.Client(vertexai=True, project="bounti-prod-322900", location='us-central1')

   # Load the JSON schema and examples
   json_schema = _load_json_schema()
   example_compositions = _load_example_compositions()

   system_prompt = f"""
You are a video rendering expert responsible for fixing a broken declarativemoviepy JSON composition.

The JSON composition failed to render with this error:
{error_message}

Your task is to analyze the error and fix the JSON composition to make it render successfully.

Common issues and fixes:
- "operands could not be broadcast together with shapes" errors: Often caused by clips with zero dimensions or mismatched image sizes. Remove problematic clips or fix their dimensions.
- Frame extraction errors: Video timestamps might be beyond video duration. Adjust timestamps or remove problematic enhanced_image clips.
- Missing assets: Remove references to assets that don't exist.
- Invalid effect parameters: Fix or remove effects with invalid parameters.

IMPORTANT: You must strictly follow the JSON schema provided below. The output MUST be a valid JSON object that conforms to the schema.

## JSON Schema for Declarative MoviePy Compositions:

{json_schema}

## Example Compositions:

{example_compositions}

Please analyze the error and return a FIXED version of the JSON composition. Only return the JSON, no explanations.
"""

   user_prompt = f"""
Here is the current JSON composition that failed to render:

{json.dumps(current_json, indent=2)}

Error encountered:
{error_message}

Please fix this JSON composition to resolve the rendering error. Return only the corrected JSON.
"""

   max_retries = 2
   for attempt in range(max_retries):
       try:
           response = client.models.generate_content(
               model="gemini-2.5-pro",
               contents=[system_prompt, user_prompt],
           )

           text_response = response.text
           # The response should be JSON, but in case it's wrapped in markdown
           match = re.search(r'```json\s*([\s\S]*?)\s*```', text_response)
           if match:
               json_str = match.group(1)
           else:
               # Robustly find JSON object
               start_index = text_response.find('{')
               end_index = text_response.rfind('}')
               if start_index != -1 and end_index != -1 and start_index < end_index:
                   json_str = text_response[start_index:end_index+1]
               else:
                   json_str = text_response

           fixed_json = json.loads(json_str)

           # Ensure required fields are present
           fixed_json.setdefault("output_path", "montage.mp4")
           fixed_json.setdefault("size", resolution or [1920, 1080])
           fixed_json.setdefault("fps", 24)

           return fixed_json

       except Exception as e:
           logging.warning(f"Attempt {attempt + 1} to fix montage JSON failed: {e}")
           if attempt + 1 == max_retries:
               logging.error(f"Failed to fix montage JSON after {max_retries} attempts.")
               raise e

   raise Exception("Failed to fix montage JSON")

def _generate_montage_json(
   analyses: list,
   asset_metadata: dict = None,
   prompt: str = None,
   length: int = None,
   resolution: list = None,
) -> dict:
    """Generates a declarative montage JSON from analyses using Gemini."""
    logging.info("Generating montage JSON from analyses via LLM")

    if not analyses:
        return {
            "output_path": "montage.mp4",
            "size": resolution or [1920, 1080],
            "fps": 24,
            "clip": {
                "type": "text",
                "text": "No assets provided to generate montage.",
                "font_size": 50,
                "color": "white",
                "duration": 5,
                "size": resolution or [1920, 1080],
            },
        }

    client = genai.Client(vertexai=True, project="bounti-prod-322900", location='us-central1')

    # Load the JSON schema and examples
    json_schema = _load_json_schema()
    example_compositions = _load_example_compositions()

    system_prompt = f"""
You are a creative video editor responsible for creating a declarative JSON definition for a video montage.
Based on the provided analysis of video and photo assets, and following the user's creative direction, you will generate a JSON object that conforms to the declarativemoviepy schema.

The output MUST be a valid JSON object. Do not include any text, code block markers, or explanations before or after the JSON object.

IMPORTANT: You must strictly follow the JSON schema provided below. The schema defines all valid clip types, effects, and properties.

## JSON Schema for Declarative MoviePy Compositions:

{json_schema}

## Key Schema Rules:
- The root object requires `output_path` and `clip` properties
- All clip types must have a `type` property matching one of the defined constants
- Common clip properties: `start`, `position`, `duration`, `effects`
- Position arrays use format: [x, y] where x/y can be numbers or strings like "center"
- Effects must match the defined effect types with their specific properties
- Use subclip effect for video segments: {{"type": "subclip", "t_start": <seconds>, "t_end": <seconds>}}

## Available Music Tracks:
- gs://real-estate-videos/montages_music/Call me crazy - Patrick Patrikios.mp3
- gs://real-estate-videos/montages_music/City lights - Patrick Patrikios.mp3
- gs://real-estate-videos/montages_music/Cruise control - Patrick Patrikios.mp3
- gs://real-estate-videos/montages_music/Down The Rabbit Hole - The Grey Room _ Density & Time.mp3
- gs://real-estate-videos/montages_music/Last laugh - Patrick Patrikios.mp3
- gs://real-estate-videos/montages_music/On The Flip - The Grey Room _ Density & Time.mp3
- gs://real-estate-videos/montages_music/Twinkle - The Grey Room _ Density & Time.mp3

## Example Compositions:

{example_compositions}

Your task is to assemble clips that conform to the schema into a coherent and engaging montage based on the user's request.
"""

    # Extract the list of provided asset URIs
    provided_assets = [asset.get("source") for asset in analyses if asset.get("source")]
    
    base_user_prompt_parts = [
        "Please generate the declarativemoviepy JSON for a new montage.",
        "CRITICAL: You MUST ONLY use assets from this provided list. Do not reference any other assets:",
        json.dumps(provided_assets, indent=2),
        "",
        "Here is the analysis of the available media assets:",
        json.dumps(analyses, indent=2),
    ]
    
    # Add asset metadata to the prompt if available
    if asset_metadata:
        base_user_prompt_parts.extend([
            "",
            "IMPORTANT: Here is the technical metadata for each asset (duration in seconds, resolution):",
            json.dumps(asset_metadata, indent=2),
            "",
            "Please use this metadata to:",
            "- Ensure video subclip effects (t_end) do not exceed the actual video duration",
            "- Choose appropriate timing for segments based on available content length",
            "- Consider the native resolution when positioning and sizing clips",
            "- Account for the actual FPS when setting timing-sensitive effects",
        ])

    if prompt:
        base_user_prompt_parts.append(f"User's creative direction: {prompt}")
    if length:
        base_user_prompt_parts.append(f"The total duration of the montage should be approximately {length} seconds.")
    
    if resolution:
        base_user_prompt_parts.append(f"The video resolution should be {resolution[0]}x{resolution[1]} pixels.")
    else:
        resolution = [1920, 1080]  # default

    validation_error_feedback = None
    last_exception = None
    max_retries = 3
    for attempt in range(max_retries):
        user_prompt_parts = base_user_prompt_parts.copy()
        if validation_error_feedback:
            user_prompt_parts.append(validation_error_feedback)
        
        user_prompt = "\n\n".join(user_prompt_parts)

        try:
            response = client.models.generate_content(
                model="gemini-2.5-pro",
                contents=[system_prompt, user_prompt],
            )

            text_response = response.text
            # The response should be JSON, but in case it's wrapped in markdown
            match = re.search(r'```json\s*([\s\S]*?)\s*```', text_response)
            if match:
                json_str = match.group(1)
            else:
                # Robustly find JSON object
                start_index = text_response.find('{')
                end_index = text_response.rfind('}')
                if start_index != -1 and end_index != -1 and start_index < end_index:
                    json_str = text_response[start_index:end_index+1]
                else:
                    json_str = text_response

            montage_json = json.loads(json_str)

            # Validate the generated JSON
            validate_composition_json(montage_json)

            # Ensure required fields are present
            montage_json.setdefault("output_path", "montage.mp4")
            montage_json.setdefault("size", resolution)
            montage_json.setdefault("fps", 24)

            return montage_json

        except SchemaValidationError as e:
            logging.warning(f"Attempt {attempt + 1} failed validation: {e}")
            last_exception = e
            validation_error_feedback = (
                "The previous JSON you generated failed schema validation. "
                f"Please correct it. The error was: {e}"
            )
        except Exception as e:
            logging.warning(f"Attempt {attempt + 1} to generate montage JSON failed: {e}")
            last_exception = e
        
        if attempt + 1 < max_retries:
            time.sleep(2 ** attempt)
        
    logging.error(f"Failed to generate montage JSON after {max_retries} attempts.")
    # Fallback to a simple error clip
    return {
        "output_path": "montage.mp4",
        "size": resolution,
        "fps": 24,
        "clip": {
            "type": "text",
            "text": f"Failed to generate montage: {last_exception}",
            "font_size": 50,
            "color": "white",
            "duration": 5,
            "size": resolution,
        },
    }

def process_assets_and_create_montage(
   assets: list,
   output_gcs_uri: str,
   prompt: str = None,
   length: int = None,
   resolution: list = None,
) -> str:
   """
   Analyzes assets, creates a montage JSON, renders the video, and uploads it to GCS.
   Returns the GCS path to the rendered video file.
   """
   try:
       # Step 1: Extract asset metadata (duration, resolution, etc.)
       logging.info("Extracting asset metadata...")
       asset_metadata = _extract_asset_metadata(assets)
       
       # Step 2: Analyze assets for content
       analyses = []
       for asset in assets:
           asset_type = asset.get("type")
           gcs_uri = asset.get("gcs_uri")
           
           if not asset_type or not gcs_uri:
               logging.warning(f"Skipping invalid asset: {asset}")
               continue
               
           if asset_type == "video":
               analysis = _analyze_video(gcs_uri)
               analyses.append(analysis)
           elif asset_type == "photo":
               analysis = _analyze_photo(gcs_uri)
               analyses.append(analysis)
           else:
               logging.warning(f"Unsupported asset type '{asset_type}' for {gcs_uri}")

       # Step 3: Generate and render with retry loop
       max_retries = 3
       last_error = None
       montage_json = None
       
       for attempt in range(max_retries):
           try:
               logging.info(f"Generation attempt {attempt + 1}/{max_retries}")
               
               if attempt == 0:
                   # First attempt: Generate new JSON
                   montage_json = _generate_montage_json(analyses, asset_metadata, prompt, length, resolution)
               else:
                   # Subsequent attempts: Fix the existing JSON based on error
                   montage_json = _fix_montage_json(montage_json, last_error, analyses, asset_metadata, prompt, length, resolution)
               
               # Step 4: Ensure output path points to video file, not JSON
               if not output_gcs_uri.startswith("gs://"):
                   raise ValueError("output_gcs_uri must be a GCS path (gs://...).")
               
               # Make sure the output path has a video extension
               if not any(output_gcs_uri.lower().endswith(ext) for ext in ['.mp4', '.avi', '.mov', '.mkv']):
                   if output_gcs_uri.endswith('.json'):
                       output_gcs_uri = output_gcs_uri.replace('.json', '.mp4')
                   else:
                       output_gcs_uri = f"{output_gcs_uri.rstrip('/')}/montage.mp4"
               
               # Update the JSON with the correct output path
               montage_json["output_path"] = output_gcs_uri
               
               # Step 5: Save the JSON composition file alongside the video
               json_gcs_uri = output_gcs_uri.rsplit('.', 1)[0] + '.json'
               montage_json_str = json.dumps(montage_json, indent=2)
               
               # Parse JSON GCS URI to upload the composition
               json_parts = json_gcs_uri[5:].split("/", 1)  # Remove 'gs://'
               json_bucket_name = json_parts[0]
               json_blob_name = json_parts[1]
               
               json_result_path = _upload_to_gcs(json_bucket_name, json_blob_name, montage_json_str)
               logging.info(f"Saved composition JSON to: {json_result_path}")
               
               # Step 6: Render the video using declarativemoviepy
               logging.info("Starting video rendering...")
               render_from_json(montage_json)
               logging.info(f"Video rendering complete. Output: {output_gcs_uri}")
               
               return output_gcs_uri
               
           except Exception as e:
               last_error = str(e)
               logging.error(f"Attempt {attempt + 1} failed: {last_error}")
               if attempt + 1 == max_retries:
                   logging.error(f"Failed after {max_retries} attempts. Final error: {last_error}")
                   raise e
       
       raise Exception("Max retries exceeded")
       
   except Exception as e:
       logging.error(f"Failed to process assets and create montage: {e}")
       raise
