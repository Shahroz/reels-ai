# Declarative MoviePy

`declarativemoviepy` is a Python library that allows you to define video compositions using a JSON file, which are then rendered using the powerful `moviepy` library. This approach separates the video structure from the rendering logic, making it easy to generate and manipulate video templates programmatically.

It's particularly useful for:
-   Automating video creation on a server.
-   Building video editing UIs where the output is a JSON definition.
-   Creating complex, template-based video compositions.

This project is built on top of `moviepy` and extends its functionality by providing a declarative interface.

## Features

-   **Declarative Syntax:** Define your entire video composition in a single JSON file.
-   **GCS Integration:** Natively handles asset downloading from and uploading to Google Cloud Storage (`gs://` URIs).
-   **Extensible:** Supports a wide range of `moviepy` clips, compositions, and effects.
-   **Easy to Use:** Render a video with a single command.

## JSON Schema

The structure of the JSON composition is defined by a schema. For a detailed reference of all available clip types, properties, and effects, please see the [JSON Schema documentation](docs/JSON_SCHEMA.md).

## Basic Example

Here is a simple example of a JSON composition that overlays a text clip on a video clip.

**`my_video.json`**
```json
{
  "output_path": "result.mp4",
  "fps": 24,
  "clip": {
    "type": "composite",
    "clips": [
      {
        "type": "video",
        "source": "path/to/your/video.mp4",
        "duration": 10
      },
      {
        "type": "text",
        "text": "Hello, Declarative World!",
        "font_size": 70,
        "color": "white",
        "start": 1,
        "duration": 8,
        "effects": [
          {
            "type": "fadein",
            "duration": 1
          },
          {
            "type": "fadeout",
            "duration": 1
          }
        ]
      }
    ]
  }
}
```

## How to Run

To render a video from a JSON file, you can run the main module from the command line:

```bash
python -m declarativemoviepy.main path/to/your/composition.json
```

This will:
1.  Parse the JSON file.
2.  Download any assets from GCS if specified.
3.  Build the `moviepy` clip.
4.  Render the video to the specified `output_path`.
5.  Upload the result to GCS if the output path is a `gs://` URI.

## Installation

Install the necessary packages using pip:

```bash
pip install -r requirements.txt
```

This will install `moviepy` and `google-cloud-storage`.