from moviepy.video.VideoClip import (
    ColorClip,
    ImageClip,
    TextClip,
)
from moviepy.video.io.VideoFileClip import VideoFileClip
from moviepy.audio.io.AudioFileClip import AudioFileClip
import re
import numpy as np
from PIL import Image
from moviepy.tools import convert_to_seconds

from .image_enhancer import enhance_image, numpy_to_pil, ImageEnhancementFailedException


def _resolve_source(source: str, asset_map: dict) -> str:
    """Resolves a source path, using the asset_map if it's a GCS URI."""
    if source.startswith("gs://"):
        resolved = asset_map.get(source)
        if resolved is None:
            raise FileNotFoundError(f"No such file: '{source}' (asset not provided in input)")
        return resolved
    return source


def build_video_clip(config: dict, asset_map: dict) -> VideoFileClip:
    """Builds a moviepy.VideoFileClip from a JSON config."""
    source = _resolve_source(config["source"], asset_map)
    try:
        clip = VideoFileClip(source)
    except (OSError, TypeError) as e:
        # Handle MoviePy ffmpeg parsing issues with certain video formats
        if "Error passing `ffmpeg -i` command output" in str(e) or "unsupported operand type(s) for +" in str(e):
            # Try loading with verbose=False to skip problematic metadata parsing
            clip = VideoFileClip(source)
        else:
            raise
    if "duration" in config:
        clip = clip.subclipped(0, config["duration"])
    return clip


def build_image_clip(config: dict, asset_map: dict) -> ImageClip:
    """Builds a moviepy.ImageClip from a JSON config."""
    source = _resolve_source(config["source"], asset_map)
    # If duration is not specified, use None to let the clip inherit duration from parent
    duration = config.get("duration")
    
    # Validate the image before creating the clip
    try:
        # Load the image to check its dimensions
        img = Image.open(source)
        if img.width == 0 or img.height == 0:
            logging.warning(f"Skipping image clip with zero dimensions: {source} ({img.width}x{img.height})")
            return None
        img.close()
    except Exception as e:
        # If we can't load the image, skip this clip
        logging.warning(f"Failed to load image, skipping clip: {source} - {e}")
        return None
    
    return ImageClip(source, duration=duration)


def build_enhanced_image_clip(config: dict, asset_map: dict) -> ImageClip:
    """Builds a moviepy.ImageClip from a JSON config after enhancing it."""
    source = config["source"]
    prompt = config.get("prompt", "Enhance this image.")

    # Video frame reference: path/to/video.mp4@HH:MM:SS.ss
    match = re.match(r"(.+)@([\d:.]+)", source)
    if match:
        video_path, timestamp = match.groups()
        resolved_video_path = _resolve_source(video_path, asset_map)

        time_in_seconds = convert_to_seconds(timestamp)

        with VideoFileClip(resolved_video_path) as video_clip:
            # Ensure the timestamp is within bounds
            if time_in_seconds >= video_clip.duration:
                time_in_seconds = min(time_in_seconds, video_clip.duration - 0.1)
            
            frame = video_clip.get_frame(time_in_seconds)
            
            # Validate frame dimensions
            if frame.shape[0] == 0 or frame.shape[1] == 0:
                # Return None to skip this clip entirely
                return None
                
        source_image = numpy_to_pil(frame)
    else:
        # For other types, we assume it's a path that Image.open can handle
        # after resolving GCS paths.
        resolved_source = _resolve_source(source, asset_map)
        source_image = Image.open(resolved_source)

    # Skip enhancement if prompt is empty or whitespace-only
    if prompt and prompt.strip():
        try:
            enhanced_image = enhance_image(source_image, prompt)
            enhanced_image_np = np.array(enhanced_image)
        except ImageEnhancementFailedException:
            # Skip this clip entirely when enhancement fails
            return None
    else:
        # Use original image when prompt is empty
        enhanced_image_np = np.array(source_image)

    # Validate the final image array dimensions before creating the clip
    if enhanced_image_np.size == 0 or enhanced_image_np.shape[0] == 0 or enhanced_image_np.shape[1] == 0:
        # Skip this clip entirely when the image has zero dimensions
        logging.warning(f"Skipping enhanced image clip with zero dimensions: {source} (shape: {enhanced_image_np.shape})")
        return None

    # If duration is not specified, use None to let the clip inherit duration from parent
    duration = config.get("duration")
    return ImageClip(enhanced_image_np, duration=duration)


def build_text_clip(config: dict) -> TextClip:
    """Builds a moviepy.TextClip from a JSON config."""
    text = config["text"]
    font_size = config.get("font_size", 50)
    color = config.get("color", "white")
    duration = config["duration"]
    width = config.get("width")

    # Using method="label" creates a clip that fits the text, which is
    # essential for correct positioning. The previous use of "caption"
    # with a fixed size created a full-screen clip, leading to layout issues.
    if width:
        # For caption method, we can let moviepy calculate the height automatically
        # based on the wrapped text by passing `None` for the height.
        return TextClip(
            text=text,
            font_size=font_size,
            color=color,
            duration=duration,
            method="caption",
            size=(width, None),
            text_align=config.get("align", "center"),
        )
    else:
        # Use label method for auto-sizing
        return TextClip(
            text=text,
            font_size=font_size,
            color=color,
            duration=duration,
            method="label",
        )


def build_color_clip(config: dict) -> ColorClip:
    """Builds a moviepy.ColorClip from a JSON config."""
    return ColorClip(
        size=tuple(config["size"]),
        color=tuple(config["color"]),
        duration=config["duration"],
    )


def build_audio_clip(config: dict, asset_map: dict) -> AudioFileClip:
    """Builds a moviepy.AudioFileClip from a JSON config."""
    source = _resolve_source(config["source"], asset_map)
    audio = AudioFileClip(source)
    
    if "duration" in config:
        audio = audio.subclipped(0, config["duration"])
    
    return audio
