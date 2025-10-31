#!/usr/bin/env python3

import json
from typing import Dict, Any, List

def generate_json_schema() -> Dict[str, Any]:
    """
    Generate JSON Schema for declarativemoviepy composition format.
    """
    
    # Base clip properties that all clips inherit
    clip_base_properties = {
        "type": {"type": "string", "description": "The type of the clip"},
        "start": {"type": "number", "description": "Time in seconds when the clip starts within a composite", "minimum": 0},
        "position": {
            "type": "array",
            "description": "Position of the clip's top-left corner within a composite",
            "prefixItems": [
                {"anyOf": [{"type": "string", "enum": ["left", "center", "right"]}, {"type": "number"}]},
                {"anyOf": [{"type": "string", "enum": ["top", "center", "bottom"]}, {"type": "number"}]}
            ],
            "minItems": 2,
            "maxItems": 2
        },
        "duration": {"type": "number", "description": "Duration of the clip in seconds", "minimum": 0},
        "effects": {
            "type": "array",
            "description": "List of effects to apply to this clip",
            "items": {"$ref": "#/$defs/effect"}
        }
    }
    
    # Effect definitions
    effect_definitions = {
        "effect": {
            "type": "object",
            "discriminator": {"propertyName": "type"},
            "oneOf": [
                {
                    "type": "object",
                    "properties": {
                        "type": {"const": "fadein"},
                        "duration": {"type": "number", "minimum": 0, "description": "Duration of the fade-in effect"}
                    },
                    "required": ["type", "duration"],
                    "additionalProperties": False
                },
                {
                    "type": "object",
                    "properties": {
                        "type": {"const": "fadeout"},
                        "duration": {"type": "number", "minimum": 0, "description": "Duration of the fade-out effect"}
                    },
                    "required": ["type", "duration"],
                    "additionalProperties": False
                },
                {
                    "type": "object",
                    "properties": {
                        "type": {"const": "speed"},
                        "factor": {"type": "number", "minimum": 0, "description": "Speed multiplier"}
                    },
                    "required": ["type", "factor"],
                    "additionalProperties": False
                },
                {
                    "type": "object",
                    "properties": {
                        "type": {"const": "multiply_speed"},
                        "factor": {"type": "number", "minimum": 0, "description": "Speed multiplier"}
                    },
                    "required": ["type", "factor"],
                    "additionalProperties": False
                },
                {
                    "type": "object",
                    "properties": {
                        "type": {"const": "multiply_volume"},
                        "factor": {"type": "number", "minimum": 0, "description": "Volume multiplier"}
                    },
                    "required": ["type", "factor"],
                    "additionalProperties": False
                },
                {
                    "type": "object",
                    "properties": {
                        "type": {"const": "audio_fadein"},
                        "duration": {"type": "number", "minimum": 0, "description": "Duration of the audio fade-in effect"}
                    },
                    "required": ["type", "duration"],
                    "additionalProperties": False
                },
                {
                    "type": "object",
                    "properties": {
                        "type": {"const": "audio_fadeout"},
                        "duration": {"type": "number", "minimum": 0, "description": "Duration of the audio fade-out effect"}
                    },
                    "required": ["type", "duration"],
                    "additionalProperties": False
                },
                {
                    "type": "object",
                    "properties": {
                        "type": {"const": "subclip"},
                        "t_start": {"type": "number", "minimum": 0, "description": "Start time for subclip"},
                        "t_end": {"type": "number", "minimum": 0, "description": "End time for subclip"}
                    },
                    "required": ["type", "t_start", "t_end"],
                    "additionalProperties": False
                },
                {
                    "type": "object",
                    "properties": {
                        "type": {"const": "resize"},
                        "width": {"type": "number", "minimum": 1, "description": "Target width"},
                        "height": {"type": "number", "minimum": 1, "description": "Target height"}
                    },
                    "required": ["type"],
                    "additionalProperties": False
                }
            ]
        }
    }
    
    # Clip type definitions
    clip_definitions = {
        "clip": {
            "type": "object",
            "discriminator": {"propertyName": "type"},
            "oneOf": [
                {
                    "type": "object",
                    "properties": {
                        **clip_base_properties,
                        "type": {"const": "video"},
                        "source": {"type": "string", "description": "Path to the video file or gs:// URI"}
                    },
                    "required": ["type", "source"],
                    "additionalProperties": False
                },
                {
                    "type": "object",
                    "properties": {
                        **clip_base_properties,
                        "type": {"const": "image"},
                        "source": {"type": "string", "description": "Path to the image file or gs:// URI"},
                        "duration": {"type": "number", "minimum": 0, "description": "Duration to display the image"}
                    },
                    "required": ["type", "source", "duration"],
                    "additionalProperties": False
                },
                {
                    "type": "object",
                    "properties": {
                        **clip_base_properties,
                        "type": {"const": "enhanced_image"},
                        "source": {"type": "string", "description": "Path to the source image or video frame reference"},
                        "prompt": {"type": "string", "description": "Text prompt for image enhancement", "default": "Enhance this image."},
                        "duration": {"type": "number", "minimum": 0, "description": "Duration to display the enhanced image"}
                    },
                    "required": ["type", "source", "duration"],
                    "additionalProperties": False
                },
                {
                    "type": "object",
                    "properties": {
                        **clip_base_properties,
                        "type": {"const": "text"},
                        "text": {"type": "string", "description": "Text content to display"},
                        "font": {"type": "string", "description": "Font name"},
                        "font_size": {"type": "number", "minimum": 1, "description": "Font size"},
                        "color": {"type": "string", "description": "Text color (name or hex)"},
                        "width": {"type": "number", "minimum": 1, "description": "Text width for wrapping"},
                        "align": {"type": "string", "enum": ["left", "center", "right"], "description": "Text alignment"},
                        "duration": {"type": "number", "minimum": 0, "description": "Duration to display the text"}
                    },
                    "required": ["type", "text", "duration"],
                    "additionalProperties": False
                },
                {
                    "type": "object",
                    "properties": {
                        **clip_base_properties,
                        "type": {"const": "color"},
                        "size": {
                            "type": "array",
                            "description": "Dimensions [width, height] in pixels",
                            "items": {"type": "integer", "minimum": 1},
                            "minItems": 2,
                            "maxItems": 2
                        },
                        "color": {
                            "type": "array",
                            "description": "RGB color values [r, g, b]",
                            "items": {"type": "integer", "minimum": 0, "maximum": 255},
                            "minItems": 3,
                            "maxItems": 3
                        },
                        "duration": {"type": "number", "minimum": 0, "description": "Duration to display the color"}
                    },
                    "required": ["type", "size", "color", "duration"],
                    "additionalProperties": False
                },
                {
                    "type": "object",
                    "properties": {
                        **clip_base_properties,
                        "type": {"const": "composite"},
                        "size": {
                            "type": "array",
                            "description": "Dimensions [width, height] in pixels",
                            "items": {"type": "integer", "minimum": 1},
                            "minItems": 2,
                            "maxItems": 2
                        },
                        "clips": {
                            "type": "array",
                            "description": "List of child clips to compose together",
                            "items": {"$ref": "#/$defs/clip"},
                            "minItems": 1
                        }
                    },
                    "required": ["type", "clips"],
                    "additionalProperties": False
                },
                {
                    "type": "object",
                    "properties": {
                        **clip_base_properties,
                        "type": {"const": "concatenate"},
                        "clips": {
                            "type": "array",
                            "description": "List of child clips to play in sequence",
                            "items": {"$ref": "#/$defs/clip"},
                            "minItems": 1
                        },
                        "transition": {
                            "type": "object",
                            "description": "Transition between clips",
                            "properties": {
                                "type": {"type": "string", "enum": ["crossfade"], "description": "Type of transition"},
                                "duration": {"type": "number", "minimum": 0, "description": "Duration of transition"}
                            },
                            "required": ["type", "duration"],
                            "additionalProperties": False
                        }
                    },
                    "required": ["type", "clips"],
                    "additionalProperties": False
                },
                {
                    "type": "object",
                    "properties": {
                        **clip_base_properties,
                        "type": {"const": "audio"},
                        "source": {"type": "string", "description": "Path to the audio file or gs:// URI"}
                    },
                    "required": ["type", "source"],
                    "additionalProperties": False
                }
            ]
        }
    }
    
    # Main schema structure
    schema = {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://declarativemoviepy.com/schema/composition.json",
        "title": "Declarative MoviePy Composition",
        "description": "JSON schema for declarativemoviepy video composition format",
        "type": "object",
        "properties": {
            "output_path": {
                "type": "string",
                "description": "Destination path for the rendered video (local path or gs:// URI)"
            },
            "size": {
                "type": "array",
                "description": "Dimensions [width, height] of the output video in pixels",
                "items": {"type": "integer", "minimum": 1},
                "minItems": 2,
                "maxItems": 2
            },
            "fps": {
                "type": "number",
                "description": "Frames per second for the output video",
                "minimum": 1
            },
            "audio": {
                "$ref": "#/$defs/clip",
                "description": "Audio track for the composition"
            },
            "clip": {
                "$ref": "#/$defs/clip",
                "description": "Root clip that defines the video content"
            }
        },
        "required": ["output_path", "clip"],
        "additionalProperties": False,
        "$defs": {
            **clip_definitions,
            **effect_definitions
        }
    }
    
    return schema

def save_schema_to_file(schema: Dict[str, Any], filename: str = "composition_schema.json"):
    """Save the generated schema to a JSON file."""
    with open(filename, 'w', encoding='utf-8') as f:
        json.dump(schema, f, indent=2, ensure_ascii=False)
    print(f"JSON Schema saved to {filename}")

def main():
    """Generate and save the JSON schema."""
    schema = generate_json_schema()
    save_schema_to_file(schema)
    
    # Also save to declarativemoviepy directory
    schema_path = "declarativemoviepy/composition_schema.json"
    save_schema_to_file(schema, schema_path)

if __name__ == "__main__":
    main()