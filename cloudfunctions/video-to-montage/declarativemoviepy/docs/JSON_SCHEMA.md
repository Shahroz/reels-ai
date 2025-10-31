# `declarativemoviepy` JSON Schema

This document outlines the JSON schema used by `declarativemoviepy` to define video compositions declaratively. The schema is designed to be comprehensive and cover most of `moviepy`'s core features in a structured and intuitive way.

## Top-Level: `Composition`

The root of a `declarativemoviepy` JSON file is the `Composition` object. It defines the overall properties of the video to be rendered.

| Property      | Type                   | Required | Description                                                                                             |
|---------------|------------------------|----------|---------------------------------------------------------------------------------------------------------|
| `output_path` | `string`               | Yes      | The destination path for the rendered video. Can be a local filesystem path or a `gs://` URI for Google Cloud Storage. |
| `size`        | `array [width, height]`| No       | The dimensions of the output video in pixels, e.g., `[1920, 1080]`. If not provided, it may be inferred from the root clip. |
| `fps`         | `number`               | No       | Frames per second for the output video. If not provided, it may be inferred from the root clip.        |
| `clip`        | `Clip`                 | Yes      | The root `Clip` object that defines the content of the video.                                           |

### Example

```json
{
  "output_path": "gs://my-bucket/videos/final_render.mp4",
  "size": [1280, 720],
  "fps": 24,
  "clip": {
    "type": "video",
    "source": "path/to/source.mp4"
  }
}
```

---

## The `Clip` Object

A `Clip` object is the fundamental building block of a composition. It represents a segment of video, an image, a piece of text, or a combination of other clips.

### Common Properties

All clip types share these common properties:

| Property    | Type               | Required | Description                                                              |
|-------------|--------------------|----------|--------------------------------------------------------------------------|
| `type`      | `string`           | Yes      | The type of the clip. See specific clip types below.                     |
| `start`     | `number`           | No       | The time in seconds when the clip should start playing within a `composite` clip. Defaults to `0`. |
| `position`  | `array`            | No       | The position of the clip's top-left corner within a `composite` clip, e.g., `["center", 100]` or `[50, 50]`. |
| `duration`  | `number`           | No       | The duration of the clip in seconds. For some types like `video`, this can be inferred. For `image` or `text`, it is often required. |
| `effects`   | `array` of `Effect`| No       | A list of effects to apply to this clip.                                 |

### Clip Type: `video`

Represents a clip from a video file.

| Property | Type     | Required | Description                                                              |
|----------|----------|----------|--------------------------------------------------------------------------|
| `source` | `string` | Yes      | Path to the video file. Can be a local path or a `gs://` URI.            |

### Clip Type: `image`

Represents a static image displayed for a certain duration.

| Property   | Type     | Required                 | Description                                                              |
|------------|------------|--------------------------|--------------------------------------------------------------------------|
| `source`   | `string`   | Yes                      | Path to the image file. Can be a local path or a `gs://` URI.            |
| `duration` | `number`   | Yes                      | How long the image should be displayed, in seconds.                      |

### Clip Type: `enhanced_image`

Represents an image that is enhanced by a generative AI model before being displayed as a clip.

| Property   | Type     | Required                 | Description                                                              |
|------------|------------|--------------------------|--------------------------------------------------------------------------|
| `source`   | `string`   | Yes                      | Path to the source image. Can be a local path, a `gs://` URI, or a video frame reference like `path/to/video.mp4@01:34`. |
| `prompt`   | `string`   | No                       | The text prompt to guide the image enhancement. Defaults to "Enhance this image.". |
| `duration` | `number`   | Yes                      | How long the enhanced image should be displayed, in seconds.                      |

### Clip Type: `text`

Represents a text overlay.

| Property      | Type     | Required | Description                                      |
|---------------|----------|----------|--------------------------------------------------|
| `text`        | `string` | Yes      | The text content to display.                     |
| `font`        | `string` | No       | The name of the font to use (e.g., "Arial").     |
| `font_size`   | `number` | No       | The size of the font.                            |
| `color`       | `string` | No       | The color of the text (e.g., "white", "#FF0000").|
| `duration`    | `number` | Yes      | How long the text should be displayed, in seconds. |

### Clip Type: `color`

Represents a solid color block.

| Property   | Type                   | Required | Description                                      |
|------------|------------------------|----------|--------------------------------------------------|
| `size`     | `array [width, height]`| Yes      | The dimensions of the color clip in pixels.      |
| `color`    | `array [r, g, b]`      | Yes      | The RGB color values, from 0 to 255.             |
| `duration` | `number`               | Yes      | How long the color block should be displayed.    |

### Clip Type: `composite`

Combines multiple clips by overlaying them. Clips are layered based on their order in the `clips` array, with later clips appearing on top.

| Property | Type            | Required | Description                                                              |
|----------|-----------------|----------|--------------------------------------------------------------------------|
| `size`   | `array [width, height]` | No | The dimensions of the composite clip. If not provided, it's often inferred from the first clip. |
| `clips`  | `array` of `Clip` | Yes      | A list of child clips to be composed together. Each child clip can have a `start` property to define its timing. |

### Clip Type: `concatenate`

Plays multiple clips one after another in sequence.

| Property | Type            | Required | Description                               |
|----------|-----------------|----------|-------------------------------------------|
| `clips`  | `array` of `Clip` | Yes      | A list of child clips to be played in sequence. |

### Clip Type: `concatenate`

Plays multiple clips one after another in sequence.

| Property | Type            | Required | Description                               |
|----------|-----------------|----------|-------------------------------------------|
| `clips`  | `array` of `Clip` | Yes      | A list of child clips to be played in sequence. |
| `transition` | `object` | No | Defines a transition between clips. |

**Transition Object:**

| Property | Type | Required | Description |
|---|---|---|---|
| `type` | `string` | Yes | The type of transition, e.g., `"crossfade"`.|
| `duration` | `number` | Yes | The duration of the transition in seconds. |

---

## The `Effect` Object

Effects modify the appearance or timing of a clip. An effect is an object with a `type` and other parameters specific to that effect.

### Common Effect: `fadein`

Fades the clip in from black.

| Property   | Type     | Required | Description                        |
|------------|----------|----------|------------------------------------|
| `duration` | `number` | Yes      | The duration of the fade-in effect. |

**Example:**
```json
{
  "type": "fadein",
  "duration": 1.5
}
```

### Common Effect: `speed`

Changes the playback speed of the clip.

| Property | Type     | Required | Description                                      |
|----------|----------|----------|--------------------------------------------------|
| `factor` | `number` | Yes      | The speed multiplier (e.g., `2.0` for 2x speed). |

**Example:**
```json
{
  "type": "speed",
  "factor": 2.0
}
```

---

## Full Example

Here is a more comprehensive example demonstrating a composition with multiple clips and effects.

```json
{
  "output_path": "final.mp4",
  "size": [1920, 1080],
  "fps": 30,
  "clip": {
    "type": "composite",
    "clips": [
      {
        "type": "video",
        "source": "background_video.mp4"
      },
      {
        "type": "text",
        "text": "Declarative MoviePy!",
        "font": "Impact",
        "font_size": 120,
        "color": "white",
        "start": 1.0,
        "duration": 5.0,
        "effects": [
          {
            "type": "fadein",
            "duration": 1.0
          }
        ]
      }
    ]
  }
}
```
