import logging
from moviepy.Clip import Clip
from moviepy.Effect import Effect
from moviepy.audio import fx as afx
from moviepy.video import fx as vfx

EFFECT_MAP = {
    # video effects
    "fadein": vfx.FadeIn,
    "fadeout": vfx.FadeOut,
    "multiply_speed": vfx.MultiplySpeed,
    "speed": vfx.MultiplySpeed,  # alias for multiply_speed
    "resize": vfx.Resize,
    "rotate": vfx.Rotate,
    "crop": vfx.Crop,
    "invert_colors": vfx.InvertColors,
    "multiply_color": vfx.MultiplyColor,
    # audio effects
    "audio_fadein": afx.AudioFadeIn,
    "audio_fadeout": afx.AudioFadeOut,
    "multiply_volume": afx.MultiplyVolume,
    "audio_normalize": afx.AudioNormalize,
    "audio_loop": afx.AudioLoop,
}


def apply_effects(clip: Clip, effects_config: list) -> Clip:
    """Applies a list of effects to a clip.

    This function iterates through a list of effect configurations,
    finds the corresponding effect class from a mapping, and applies it
    to the clip with the specified parameters.

    Args:
        clip: The moviepy Clip object to apply effects to.
        effects_config: A list of dictionaries, where each dictionary
                        represents an effect's configuration. It must
                        contain a 'type' key, and other keys as parameters
                        for the effect.

    Returns:
        A moviepy.Clip.Clip object with the effects applied.
    """
    effects: list[Effect] = []
    for effect_config in effects_config:
        config = effect_config.copy()
        effect_type = config.pop("type")
        
        # Handle subclip specially since it's a method, not an effect
        if effect_type == "subclip":
            t_start = config.get("t_start", 0)
            t_end = config.get("t_end", None)
            
            # Validate and clamp t_end to clip duration to prevent errors
            if t_end is not None and hasattr(clip, 'duration') and clip.duration is not None:
                if t_end > clip.duration:
                    logging.warning(f"Subclip t_end ({t_end}) exceeds clip duration ({clip.duration}). Clamping to clip duration.")
                    t_end = clip.duration
            
            clip = clip.subclipped(t_start, t_end)
            continue
            
        effect_class = EFFECT_MAP.get(effect_type)

        if not effect_class:
            raise ValueError(f"Unknown effect type '{effect_type}'")

        # The rest of the config are parameters for the effect
        effects.append(effect_class(**config))

    if not effects:
        return clip

    return clip.with_effects(effects)