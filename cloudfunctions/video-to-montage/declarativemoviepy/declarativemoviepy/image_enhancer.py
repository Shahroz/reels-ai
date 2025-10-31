from PIL import Image
from io import BytesIO
import numpy as np
import logging
import hashlib
import os
import pickle
from pathlib import Path
import base64
import requests
import time
import google.auth
import google.auth.transport.requests

class ImageEnhancementFailedException(Exception):
    """Raised when image enhancement fails and the clip should be skipped."""
    pass

def _get_cache_dir() -> Path:
    """Get or create the cache directory for enhanced images."""
    cache_dir = Path.home() / ".declarativemoviepy" / "enhanced_images_cache"
    cache_dir.mkdir(parents=True, exist_ok=True)
    return cache_dir

def _resize_to_max_1080_png(image: Image.Image) -> Image.Image:
    """Resizes image to max 1080px on the longest side."""
    max_size = 1080
    if image.width > max_size or image.height > max_size:
        if image.width > image.height:
            new_width = max_size
            new_height = int(max_size * image.height / image.width)
        else:
            new_height = max_size
            new_width = int(max_size * image.width / image.height)
        image = image.resize((new_width, new_height), Image.Resampling.LANCZOS)
    return image

def _get_image_hash(image: Image.Image) -> str:
    """Generate a hash for the image content."""
    # Convert image to bytes for hashing
    img_bytes = BytesIO()
    image.save(img_bytes, format='PNG')
    img_bytes.seek(0)
    return hashlib.md5(img_bytes.getvalue()).hexdigest()

def _get_cache_key(image: Image.Image, prompt: str) -> str:
    """Generate a cache key from image content and prompt."""
    image_hash = _get_image_hash(image)
    prompt_hash = hashlib.md5(prompt.encode('utf-8')).hexdigest()
    return f"{image_hash}_{prompt_hash}"

def _load_cached_image(cache_key: str) -> Image.Image:
    """Load a cached enhanced image if it exists."""
    cache_dir = _get_cache_dir()
    cache_file = cache_dir / f"{cache_key}.pkl"
    
    if cache_file.exists():
        try:
            with open(cache_file, 'rb') as f:
                cached_data = pickle.load(f)
                return Image.open(BytesIO(cached_data))
        except Exception as e:
            logging.warning(f"Failed to load cached image {cache_key}: {e}")
            # Remove corrupted cache file
            try:
                cache_file.unlink()
            except:
                pass
    return None

def _save_cached_image(cache_key: str, image: Image.Image):
    """Save an enhanced image to cache."""
    try:
        cache_dir = _get_cache_dir()
        cache_file = cache_dir / f"{cache_key}.pkl"
        
        # Convert image to bytes for storage
        img_bytes = BytesIO()
        image.save(img_bytes, format='PNG')
        img_bytes.seek(0)
        
        with open(cache_file, 'wb') as f:
            pickle.dump(img_bytes.getvalue(), f)
            
        logging.info(f"Cached enhanced image to {cache_file}")
    except Exception as e:
        logging.warning(f"Failed to cache image {cache_key}: {e}")

def enhance_image(image: Image.Image, prompt: str) -> Image.Image:
    """
    Enhances an image using a generative AI model with local caching.

    Args:
        image: A PIL Image object.
        prompt: The enhancement instruction.

    Returns:
        A PIL Image object of the enhanced image.
        
    Raises:
        ImageEnhancementFailedException: When enhancement fails and clip should be skipped.
    """
    # Generate cache key
    cache_key = _get_cache_key(image, prompt)
    
    # Try to load from cache first
    cached_image = _load_cached_image(cache_key)
    if cached_image is not None:
        logging.info(f"Using cached enhanced image: {cache_key}")
        return cached_image
    
    # If not in cache, enhance the image
    try:
        logging.info(f"Enhancing image with Vertex AI model: {cache_key}")

        processed_image = _resize_to_max_1080_png(image)

        buffered = BytesIO()
        processed_image.save(buffered, format="PNG")
        img_bytes = buffered.getvalue()
        encoded_image = base64.b64encode(img_bytes).decode('utf-8')

        project_id = os.environ.get("GOOGLE_CLOUD_PROJECT", "bounti-prod-322900")
        model_id = "gemini-2.5-flash-image-preview"

        url = f"https://aiplatform.googleapis.com/v1/projects/{project_id}/locations/global/publishers/google/models/{model_id}:streamGenerateContent"

        request_body = {
            "contents": [{
                "role": "user",
                "parts": [
                    {"inlineData": {"mimeType": "image/png", "data": encoded_image}},
                    {"text": prompt}
                ]
            }],
            "generationConfig": {
                "temperature": 1,
                "maxOutputTokens": 32768,
                "responseModalities": ["TEXT", "IMAGE"],
                "topP": 0.95
            },
            "safetySettings": [
                {"category": "HARM_CATEGORY_HATE_SPEECH", "threshold": "BLOCK_NONE"},
                {"category": "HARM_CATEGORY_DANGEROUS_CONTENT", "threshold": "BLOCK_NONE"},
                {"category": "HARM_CATEGORY_SEXUALLY_EXPLICIT", "threshold": "BLOCK_NONE"},
                {"category": "HARM_CATEGORY_HARASSMENT", "threshold": "BLOCK_NONE"},
                {"category": "HARM_CATEGORY_IMAGE_HATE", "threshold": "BLOCK_NONE"},
                {"category": "HARM_CATEGORY_IMAGE_DANGEROUS_CONTENT", "threshold": "BLOCK_NONE"},
                {"category": "HARM_CATEGORY_IMAGE_HARASSMENT", "threshold": "BLOCK_NONE"},
                {"category": "HARM_CATEGORY_IMAGE_SEXUALLY_EXPLICIT", "threshold": "BLOCK_NONE"}
            ]
        }

        credentials, _ = google.auth.default(scopes=['https://www.googleapis.com/auth/cloud-platform'])
        auth_req = google.auth.transport.requests.Request()

        last_error = None
        for attempt in range(3):
            try:
                credentials.refresh(auth_req)
                headers = {
                    "Authorization": f"Bearer {credentials.token}",
                    "Content-Type": "application/json; charset=utf-8",
                }

                response = requests.post(url, headers=headers, json=request_body, timeout=60)

                if response.status_code == 200:
                    api_responses = response.json()
                    for resp_item in api_responses:
                        for candidate in resp_item.get("candidates", []):
                            for part in candidate.get("content", {}).get("parts", []):
                                if "inlineData" in part and part["inlineData"].get("data"):
                                    image_data = part["inlineData"]["data"]
                                    decoded_bytes = base64.b64decode(image_data)
                                    enhanced_image = Image.open(BytesIO(decoded_bytes))
                                    _save_cached_image(cache_key, enhanced_image)
                                    return enhanced_image
                    last_error = ImageEnhancementFailedException(f"AI model returned response without image data: {response.text}")
                    continue
                else:
                    last_error = ImageEnhancementFailedException(f"API request failed on attempt {attempt + 1} with status {response.status_code}: {response.text}")

            except requests.exceptions.RequestException as e:
                last_error = ImageEnhancementFailedException(f"Network error on attempt {attempt + 1}: {e}")

            if attempt < 2:
                logging.warning(f"{last_error} Retrying in 2 seconds...")
                time.sleep(2)

        raise last_error

    except Exception as e:
        if not isinstance(e, ImageEnhancementFailedException):
            logging.warning(f"Image enhancement failed with unexpected error: {str(e)}. Skipping this enhanced image clip.")
            raise ImageEnhancementFailedException(f"Enhancement failed: {str(e)}")
        else:
            logging.warning(f"Image enhancement failed: {str(e)}. Skipping this enhanced image clip.")
            raise e

def numpy_to_pil(np_array: np.ndarray) -> Image.Image:
    """Converts a NumPy array (frame) to a PIL Image."""
    return Image.fromarray(np_array)
