"""
Validates the declarativemoviepy JSON schema.
"""

class SchemaValidationError(ValueError):
    """Custom exception for schema validation errors."""
    pass

def _validate_clip(clip_data: dict):
    """Recursively validates a clip object."""
    if not isinstance(clip_data, dict):
        raise SchemaValidationError("Clip configuration must be a dictionary.")

    clip_type = clip_data.get("type")
    if not clip_type:
        raise SchemaValidationError("Each clip must have a 'type'.")

    # Common properties check
    if "start" in clip_data and not isinstance(clip_data["start"], (int, float)):
        raise SchemaValidationError(f"Clip 'start' must be a number for clip type {clip_type}.")
    if "duration" in clip_data and not isinstance(clip_data["duration"], (int, float)):
        raise SchemaValidationError(f"Clip 'duration' must be a number for clip type {clip_type}.")
    if "position" in clip_data and not isinstance(clip_data["position"], list):
        raise SchemaValidationError(f"Clip 'position' must be a list for clip type {clip_type}.")

    # Type-specific checks
    if clip_type == "video":
        if "source" not in clip_data:
            raise SchemaValidationError("Video clip must have a 'source'.")
    elif clip_type == "audio":
        if "source" not in clip_data:
            raise SchemaValidationError("Audio clip must have a 'source'.")
    elif clip_type == "image":
        if "source" not in clip_data:
            raise SchemaValidationError("Image clip must have a 'source'.")
        if "duration" not in clip_data:
            raise SchemaValidationError("Image clip must have a 'duration'.")
    elif clip_type == "enhanced_image":
        if "source" not in clip_data:
            raise SchemaValidationError("Enhanced image clip must have a 'source'.")
        if "duration" not in clip_data:
            raise SchemaValidationError("Enhanced image clip must have a 'duration'.")
    elif clip_type == "text":
        if "text" not in clip_data:
            raise SchemaValidationError("Text clip must have 'text'.")
        if "duration" not in clip_data:
            raise SchemaValidationError("Text clip must have a 'duration'.")
    elif clip_type == "color":
        if "size" not in clip_data or not isinstance(clip_data["size"], list):
            raise SchemaValidationError("Color clip must have a 'size' array.")
        if "color" not in clip_data or not isinstance(clip_data["color"], list):
            raise SchemaValidationError("Color clip must have a 'color' array.")
        if "duration" not in clip_data:
            raise SchemaValidationError("Color clip must have a 'duration'.")
    elif clip_type in ["composite", "concatenate"]:
        if "clips" not in clip_data or not isinstance(clip_data["clips"], list):
            raise SchemaValidationError(f"{clip_type} clip must have a 'clips' array.")
        for sub_clip in clip_data["clips"]:
            _validate_clip(sub_clip)
    else:
        allowed_clip_types = [
            "video", "audio", "image", "enhanced_image",
            "text", "color", "composite", "concatenate"
        ]
        if clip_type not in allowed_clip_types:
            raise SchemaValidationError(f"Unknown clip type: '{clip_type}'")

    if "effects" in clip_data:
        if not isinstance(clip_data["effects"], list):
            raise SchemaValidationError("'effects' must be a list.")
        for effect in clip_data["effects"]:
            if not isinstance(effect, dict) or "type" not in effect:
                raise SchemaValidationError("Each effect must be a dictionary with a 'type'.")

def validate_composition_json(data: dict):
    """
    Validates a declarativemoviepy composition JSON.
    Raises SchemaValidationError on failure.
    """
    if not isinstance(data, dict):
        raise SchemaValidationError("Top-level configuration must be a dictionary.")

    if "output_path" not in data:
        raise SchemaValidationError("Composition must have 'output_path'.")
    
    if "clip" not in data:
        raise SchemaValidationError("Composition must have a root 'clip' object.")
        
    _validate_clip(data["clip"])