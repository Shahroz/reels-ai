import os
import shutil
import tempfile
import logging
import re
from typing import Dict, Any, List, Tuple, Optional

from google.cloud import storage
from google.auth import default
from moviepy.video.io.VideoFileClip import VideoFileClip
from PIL import Image

# Setup logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')


def is_gcs_path(path: str) -> bool:
    """Checks if a given path is a GCS URI."""
    return path.startswith("gs://")


def parse_gcs_uri(uri: str) -> Tuple[str, str]:
    """Parses a GCS URI into bucket and blob path."""
    if not is_gcs_path(uri):
        raise ValueError("Expected GCS URI starting with gs://")
    _, rest = uri.split("gs://", 1)
    bucket, *path = rest.split("/", 1)
    blob_path = path[0] if path else ""
    return bucket, blob_path


def download_gcs_file(gcs_uri: str, local_dir: str) -> str:
    """
    Downloads a file from GCS to a specified local directory.
    
    Args:
        gcs_uri: The full GCS URI (e.g., "gs://bucket/path/to/file.txt").
        local_dir: The local directory to download the file into.

    Returns:
        The full local path to the downloaded file.
    """
    if not is_gcs_path(gcs_uri):
        raise ValueError(f"Not a GCS URI: {gcs_uri}")
    
    # Handle video frame references: path/to/video.mp4@HH:MM:SS
    # For downloading, we need to strip the timestamp suffix
    base_gcs_uri = gcs_uri
    match = re.match(r"(.+)@([\d:.]+)", gcs_uri)
    if match:
        base_gcs_uri = match.group(1)
    
    bucket_name, blob_path = parse_gcs_uri(base_gcs_uri)
    if not blob_path:
        raise ValueError(f"GCS URI does not point to a file: {base_gcs_uri}")

    try:
        # Check for custom service account path first
        service_account_path = os.environ.get('GCP_SERVICE_ACCOUNT_PATH')
        if service_account_path and os.path.exists(service_account_path):
            client = storage.Client.from_service_account_json(service_account_path)
        else:
            credentials, project = default()
            client = storage.Client(credentials=credentials, project=project)
    except Exception as e:
        # Fall back to default client initialization
        logging.warning(f"Could not load credentials, trying default client: {e}")
        client = storage.Client()
    bucket = client.bucket(bucket_name)
    blob = bucket.blob(blob_path)

    if not blob.exists(client):
        raise FileNotFoundError(f"GCS object not found: {base_gcs_uri}")

    # Create the local directory if it doesn't exist
    os.makedirs(local_dir, exist_ok=True)
    
    # Construct local path
    local_filename = os.path.basename(blob_path)
    local_path = os.path.join(local_dir, local_filename)

    blob.download_to_filename(local_path)
    logging.info(f"Downloaded {base_gcs_uri} to {local_path}")
    return local_path


def upload_local_file_to_gcs(local_path: str, gcs_uri: str):
    """
    Uploads a local file to a GCS URI.

    Args:
        local_path: The path to the local file to upload.
        gcs_uri: The destination GCS URI.
    """
    if not is_gcs_path(gcs_uri):
        raise ValueError(f"Not a GCS URI for destination: {gcs_uri}")

    bucket_name, blob_path = parse_gcs_uri(gcs_uri)
    try:
        # Check for custom service account path first
        service_account_path = os.environ.get('GCP_SERVICE_ACCOUNT_PATH')
        if service_account_path and os.path.exists(service_account_path):
            client = storage.Client.from_service_account_json(service_account_path)
        else:
            credentials, project = default()
            client = storage.Client(credentials=credentials, project=project)
    except Exception as e:
        # Fall back to default client initialization
        logging.warning(f"Could not load credentials, trying default client: {e}")
        client = storage.Client()
    bucket = client.bucket(bucket_name)
    blob = bucket.blob(blob_path)
    blob.upload_from_filename(local_path)
    logging.info(f"Uploaded {local_path} to {gcs_uri}")


def get_video_metadata(video_path: str) -> Dict[str, Any]:
    """
    Extracts metadata from a video file.
    
    Returns:
        Dict containing duration, width, height, fps
    """
    try:
        with VideoFileClip(video_path) as clip:
            return {
                "duration": clip.duration,
                "width": clip.w if clip.w else 0,
                "height": clip.h if clip.h else 0,
                "fps": clip.fps if clip.fps else 24,
            }
    except Exception as e:
        logging.warning(f"Failed to extract video metadata from {video_path}: {e}")
        return {"duration": 0, "width": 0, "height": 0, "fps": 24}


def get_image_metadata(image_path: str) -> Dict[str, Any]:
    """
    Extracts metadata from an image file.
    
    Returns:
        Dict containing width, height
    """
    try:
        with Image.open(image_path) as img:
            return {
                "width": img.width,
                "height": img.height,
            }
    except Exception as e:
        logging.warning(f"Failed to extract image metadata from {image_path}: {e}")
        return {"width": 0, "height": 0}


class AssetManager:
    """
    Manages downloading GCS assets for a composition and cleaning them up.
    
    It finds all "source" keys in a dictionary, downloads the GCS URIs,
    and provides a mapping to their local paths.
    """
    def __init__(self, composition: Dict[str, Any]):
        self.composition = composition
        self.temp_dir = tempfile.mkdtemp(prefix="declarativemoviepy_assets_")
        self.local_asset_map: Dict[str, str] = {}
        self.asset_metadata: Dict[str, Dict[str, Any]] = {}
        self._download_assets()

    def _find_asset_uris(self) -> List[str]:
        """Recursively finds all 'source' URIs in the composition."""
        uris = []
        
        def recurse(obj):
            if isinstance(obj, dict):
                for key, value in obj.items():
                    if key == "source" and isinstance(value, str) and is_gcs_path(value):
                        # Handle video frame references: path/to/video.mp4@HH:MM:SS
                        match = re.match(r"(.+)@([\d:.]+)", value)
                        if match:
                            # Extract just the base video path for downloading
                            base_uri = match.groups()[0]
                            uris.append(base_uri)
                        else:
                            uris.append(value)
                    else:
                        recurse(value)
            elif isinstance(obj, list):
                for item in obj:
                    recurse(item)

        recurse(self.composition)
        return sorted(list(set(uris))) # Return unique, sorted URIs for deterministic order

    def _download_assets(self):
        """Downloads all found GCS assets to the temporary directory."""
        asset_uris = self._find_asset_uris()
        if asset_uris:
            logging.info(f"Found {len(asset_uris)} GCS assets to download.")
        for uri in asset_uris:
            try:
                local_path = download_gcs_file(uri, self.temp_dir)
                self.local_asset_map[uri] = local_path
                
                # Extract metadata based on file extension
                if local_path.lower().endswith(('.mp4', '.avi', '.mov', '.mkv', '.webm')):
                    metadata = get_video_metadata(local_path)
                    self.asset_metadata[uri] = {"type": "video", **metadata}
                elif local_path.lower().endswith(('.jpg', '.jpeg', '.png', '.gif', '.bmp', '.tiff', '.webp')):
                    metadata = get_image_metadata(local_path)
                    self.asset_metadata[uri] = {"type": "image", **metadata}
                else:
                    self.asset_metadata[uri] = {"type": "unknown"}
                    
                logging.info(f"Asset {uri} metadata: {self.asset_metadata[uri]}")
            except Exception as e:
                logging.error(f"Failed to download asset {uri}: {e}")
                # For robustness, we log and continue. A caller can check the map.
                
    def get_local_path(self, gcs_uri: str) -> str | None:
        """Returns the local path for a given GCS URI if it was downloaded."""
        # Handle video frame references by looking up the base video path
        match = re.match(r"(.+)@([\d:.]+)", gcs_uri)
        if match:
            base_uri = match.groups()[0]
            return self.local_asset_map.get(base_uri)
        return self.local_asset_map.get(gcs_uri)
    
    def get_asset_metadata(self, gcs_uri: str) -> Optional[Dict[str, Any]]:
        """Returns metadata for a given GCS URI if it was downloaded."""
        # Handle video frame references by looking up the base video path
        match = re.match(r"(.+)@([\d:.]+)", gcs_uri)
        if match:
            base_uri = match.groups()[0]
            return self.asset_metadata.get(base_uri)
        return self.asset_metadata.get(gcs_uri)

    def cleanup(self):
        """Removes the temporary directory and all downloaded assets."""
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
            logging.info(f"Cleaned up temporary asset directory: {self.temp_dir}")

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.cleanup()