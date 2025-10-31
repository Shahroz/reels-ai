from moviepy.Clip import Clip

from .clip_handlers import (
    build_audio_clip,
    build_color_clip,
    build_image_clip,
    build_text_clip,
    build_video_clip,
    build_enhanced_image_clip,
)
from .composition_handlers import build_composite_clip, build_concatenate_clip
from .effect_handlers import apply_effects


def build_clip_from_json(clip_config: dict, asset_map: dict) -> Clip:
    """
    Parses a JSON clip configuration and builds a corresponding moviepy Clip object.

    This function acts as a dispatcher, forwarding the configuration to the
    appropriate builder based on the 'type' key. After building the clip,
    it applies any specified effects.

    Args:
        clip_config: A dictionary representing the clip's configuration.
        asset_map: A dictionary mapping GCS URIs to local file paths.

    Returns:
        A moviepy.Clip.Clip object.
    """
    clip_type = clip_config["type"]
    if clip_type == "video":
        clip = build_video_clip(clip_config, asset_map)
    elif clip_type == "audio":
        clip = build_audio_clip(clip_config, asset_map)
    elif clip_type == "image":
        clip = build_image_clip(clip_config, asset_map)
    elif clip_type == "enhanced_image":
        clip = build_enhanced_image_clip(clip_config, asset_map)
        # Handle the case where enhanced_image_clip returns None (failed enhancement)
        if clip is None:
            return None
    elif clip_type == "text":
        clip = build_text_clip(clip_config)
    elif clip_type == "color":
        clip = build_color_clip(clip_config)
    elif clip_type == "composite":
        clip = build_composite_clip(clip_config, asset_map, build_clip_from_json)
    elif clip_type == "concatenate":
        clip = build_concatenate_clip(clip_config, asset_map, build_clip_from_json)
    else:
        raise ValueError(f"Unknown clip type: {clip_type}")

    # Only apply effects if clip is not None
    if clip is not None and "effects" in clip_config:
        clip = apply_effects(clip, clip_config["effects"])

    return clip
