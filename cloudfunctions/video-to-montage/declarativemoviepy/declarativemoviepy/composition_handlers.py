from typing import Callable
import numpy as np

from moviepy.Clip import Clip
from moviepy.video.VideoClip import VideoClip
from moviepy.video import fx as vfx
from moviepy.video.compositing.CompositeVideoClip import (
   CompositeVideoClip,
   concatenate_videoclips,
)


def build_composite_clip(
    config: dict, asset_map: dict, interpreter_func: Callable[[dict, dict], Clip]
) -> CompositeVideoClip:
    """Builds a moviepy.CompositeVideoClip from a JSON config."""
    child_clips = []
    for c_config in config["clips"]:
        child_clip = interpreter_func(c_config, asset_map)
        # Skip None clips (failed enhanced images)
        if child_clip is None:
            continue
        if "start" in c_config:
            child_clip = child_clip.with_start(c_config["start"])
        if "position" in c_config:
            child_clip = child_clip.with_position(tuple(c_config["position"]))
        child_clips.append(child_clip)

    # Determine composite size
    size = None
    if "size" in config:
        size = tuple(config["size"])
    else:
        # Auto-determine size from the first clip (usually background)
        if child_clips and hasattr(child_clips[0], 'size'):
            size = child_clips[0].size
            print(f"Auto-determined composite size from first clip: {size}")
    
    composite = CompositeVideoClip(child_clips, size=size)

    if "duration" in config:
        composite = composite.with_duration(config["duration"])
    else:
        # Try to determine duration from child clips that have duration
        durations = [clip.duration for clip in child_clips if clip.duration is not None]
        if durations:
            # Use the maximum duration from clips that have duration set
            composite = composite.with_duration(max(durations))

    return composite


def build_concatenate_clip(
   config: dict, asset_map: dict, interpreter_func: Callable[[dict, dict], Clip]
) -> VideoClip:
    """Builds a concatenated moviepy.VideoClip from a JSON config."""
    child_clips = [interpreter_func(c, asset_map) for c in config["clips"]]
    # Filter out None clips (failed enhanced images)
    child_clips = [clip for clip in child_clips if clip is not None]

    transition_config = config.get("transition")

    if (
        transition_config
        and transition_config.get("type") == "crossfade"
        and transition_config.get("duration", 0) > 0
    ):
        td = transition_config["duration"]
        composite_clips = []
        current_time = 0
        for i, clip in enumerate(child_clips):
            effects = []
            # Add CrossFadeOut to all clips except the last
            if i < len(child_clips) - 1:
                effects.append(vfx.CrossFadeOut(td))
            # Add CrossFadeIn to all clips except the first
            if i > 0:
                effects.append(vfx.CrossFadeIn(td))
            if effects:
                clip = clip.with_effects(effects)
            composite_clips.append(clip.with_start(current_time))
            # Calculate next start time with overlap for crossfade
            if i < len(child_clips) - 1:
                current_time += clip.duration - td
            else:
                current_time += clip.duration
        # Create composite with auto-determined size
        size = None
        if composite_clips and hasattr(composite_clips[0], 'size'):
            size = composite_clips[0].size
        concatenated = CompositeVideoClip(
            composite_clips, size=size, bg_color=None
        )
        # Fix masks created by CrossFade effects to have precomputed attribute
        for clip in composite_clips:
            if hasattr(clip, 'mask') and clip.mask is not None:
                if not hasattr(clip.mask, 'precomputed'):
                    clip.mask.precomputed = {}
    else:
        concatenated = concatenate_videoclips(child_clips)

    if "duration" in config:
        concatenated = concatenated.with_duration(config["duration"])

    return concatenated
