from google import genai
from google.genai import types
from PIL import Image
from io import BytesIO
import os

project_id = os.getenv('GOOGLE_CLOUD_PROJECT')
client = genai.Client(vertexai=True, project=project_id, location='global')

prompt = (
    "Create a picture of a nano banana dish in a fancy restaurant with a Gemini theme"
)

response = client.models.generate_content(
    model="gemini-2.5-flash-image-preview",
    contents=[prompt],
)

for part in response.candidates[0].content.parts:
    if part.text is not None:
        print(part.text)
    elif part.inline_data is not None:
        image = Image.open(BytesIO(part.inline_data.data))
        image.save("generated_image.png")