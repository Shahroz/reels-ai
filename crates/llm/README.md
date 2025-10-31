# LLM Crate

This crate provides unified interfaces for interacting with various Large Language Models (LLMs) from different vendors. It supports both simple text generation and strongly-typed structured output generation, incorporating features like automatic schema generation, few-shot examples, retries, and model fallback.

## Core Functions

### 1. `llm` - Simple Text Generation

This function is used for general-purpose text generation tasks where the expected output is a simple string.

**Signature:**

```rust
pub async fn llm(
    debug_prompt_enabled: bool, // If true, prints the prompt and response to stdout
    prompt: &str,              // The input prompt for the LLM
    models: Vec<VendorModel>, // A list of models to try (fallback in order)
    retries: usize,             // Number of retries per model
) -> anyhow::Result<String>     // The generated text or an error
```

**Parameters:**

*   `debug_prompt_enabled`: Set to `true` to print the full prompt sent to the LLM and the raw response received.
*   `prompt`: The textual prompt to send to the LLM.
*   `models`: A `Vec<VendorModel>` specifying which model(s) to use. The function will try them in the order provided, falling back to the next one if a call fails.
*   `retries`: The number of times to retry calling *each* model in the `models` list before giving up on that specific model.

**Example:**

```rust
use llm::llm_typed_unified::{llm, VendorModel};
use llm::vendors::openai::openai_model::OpenAIModel;

#[tokio::main]
asyn fn main() -> anyhow::Result<()> {
    let prompt = "Write a short poem about Rust programming.";
    let models = vec![
        VendorModel::OpenAI(OpenAIModel::Gpt4o), // Try GPT-4o first
        VendorModel::OpenAI(OpenAIModel::Gpto3Mini), // Fallback to GPT-3.5 Turbo Mini
    ];
    let retries = 2;
    let debug_mode = false;

    match llm(debug_mode, prompt, models, retries).await {
        Ok(response) => {
            println!("LLM Response:\n{}", response);
        }
        Err(e) => {
            eprintln!("LLM call failed: {}", e);
        }
    }
    Ok(())
}
```

### 2. `llm_typed` - Structured Output Generation

This function is designed for tasks where you expect the LLM to return data that conforms to a specific Rust struct or enum. It automatically generates a schema, includes few-shot examples, calls the LLM, validates the response against the schema, and deserializes it into your target type `T`.

**Signature:**

```rust
pub async fn llm_typed<T>(
    prompt: String,           // The core task prompt
    models: Vec<VendorModel>, // Models to try (fallback in order)
    retries: usize,           // Retries per model
    format: Option<OutputFormat>, // Desired output format (Json, Yaml, JsonCData)
    debug_mode: bool,       // If true, prints the full prompt and intermediate steps
) -> anyhow::Result<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned + schemars::JsonSchema + FewShotsOutput<T>,
```

**Parameters:**

*   `prompt`: The main instruction or question for the LLM.
*   `models`: A `Vec<VendorModel>` specifying the models to attempt.
*   `retries`: Number of retries per model.
*   `format`: An `Option<OutputFormat>` enum (`Json`, `Yaml`, `JsonCData`) specifying the desired format for the LLM's response. Defaults to `Json` if `None`.
*   `debug_mode`: Set to `true` to see the constructed prompt (including schema and examples) and validation steps.

**Generic Type `T` Constraints:**

The type `T` you want the LLM to return *must* satisfy these trait bounds:

*   `serde::Serialize` and `serde::de::DeserializeOwned`: For serializing examples and deserializing the LLM response.
*   `schemars::JsonSchema`: To automatically generate a JSON schema describing the structure of `T`. This schema is included in the prompt to guide the LLM.
*   `FewShotsOutput<T>`: To provide example instances of `T`. These examples are also included in the prompt to show the LLM the expected format and style.

## Providing Examples (`FewShotsOutput` Trait)

To help the LLM understand the desired output structure and format for `llm_typed`, you need to implement the `FewShotsOutput<T>` trait for your target type `T`.

**Trait Definition:**

```rust
use llm::few_shots_traits::FewShotsOutput;

pub trait FewShotsOutput<T> {
    fn few_shots() -> Vec<T>;
}
```

**Implementation:**

The `few_shots` function should return a `Vec<T>` containing one or more representative examples of the data structure you expect.

**Example Implementation:**

Let's say you want the LLM to extract information into this struct:

```rust
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use llm::few_shots_traits::FewShotsOutput;

#[derive(Serialize, Deserialize, JsonSchema, Debug)] // Needed traits
struct UserInfo {
    name: String,
    age: u32,
    city: String,
}

// Implement FewShotsOutput for UserInfo
impl FewShotsOutput<UserInfo> for UserInfo {
    fn few_shots() -> Vec<UserInfo> {
        vec![
            UserInfo {
                name: "Alice".to_string(),
                age: 30,
                city: "New York".to_string(),
            },
            UserInfo {
                name: "Bob".to_string(),
                age: 25,
                city: "San Francisco".to_string(),
            },
        ]
    }
}
```

## Full `llm_typed` Example

This example combines defining a struct, implementing `FewShotsOutput`, and calling `llm_typed`.

```rust
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use llm::few_shots_traits::FewShotsOutput;
use llm::llm_typed_unified::{llm_typed, VendorModel, OutputFormat};
use llm::vendors::openai::openai_model::OpenAIModel;

// 1. Define the target struct
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)] // Add Clone if needed by few_shots
struct AnalysisResult {
    sentiment: String, // e.g., "Positive", "Negative", "Neutral"
    keywords: Vec<String>,
    confidence: f32, // 0.0 to 1.0
}

// 2. Implement FewShotsOutput
impl FewShotsOutput<AnalysisResult> for AnalysisResult {
    fn few_shots() -> Vec<AnalysisResult> {
        vec![
            AnalysisResult {
                sentiment: "Positive".to_string(),
                keywords: vec!["great".to_string(), "excellent".to_string()],
                confidence: 0.95,
            },
            AnalysisResult {
                sentiment: "Negative".to_string(),
                keywords: vec!["terrible".to_string(), "awful".to_string()],
                confidence: 0.88,
            },
        ]
    }
}

#[tokio::main]
asyn fn main() -> anyhow::Result<()> {
    let text_to_analyze = "This new library is amazing, truly fantastic work!";
    let prompt = format!("Analyze the sentiment of the following text and extract keywords: '{}'", text_to_analyze);

    let models = vec![
        VendorModel::OpenAI(OpenAIModel::Gpt4oMini), // Use a capable model for structured output
    ];
    let retries = 1;
    let debug_mode = false; // Set to true to see the full prompt

    // 3. Call llm_typed
    match llm_typed::<AnalysisResult>(
        prompt,
        models,
        retries,
        Some(OutputFormat::Json), // Explicitly request JSON
        debug_mode,
    ).await {
        Ok(result) => {
            println!("Analysis Result: {:#?}", result);
            // Example Output:
            // Analysis Result: AnalysisResult {
            //     sentiment: "Positive",
            //     keywords: [
            //         "amazing",
            //         "fantastic",
            //         "library",
            //         "work",
            //     ],
            //     confidence: 0.98,
            // }
        }
        Err(e) => {
            eprintln!("Typed LLM call failed: {}", e);
        }
    }

    Ok(())
}
```

## Supported Models (`VendorModel` Enum)

The `VendorModel` enum allows specifying models from different providers:

*   **OpenAI:** `VendorModel::OpenAI(OpenAIModel::Gpt4o)`, `VendorModel::OpenAI(OpenAIModel::Gpto3Mini)`, etc.
*   **Gemini:** `VendorModel::Gemini(GeminiModel::Gemini15Flash)`, `VendorModel::Gemini(GeminiModel::Gemini15Pro)`, etc.
*   **Claude:** `VendorModel::Claude(ClaudeModel::Claude35SonnetLatest)`, `VendorModel::Claude(ClaudeModel::Claude3Haiku)`, etc.
*   **Replicate:** `VendorModel::Replicate(ReplicateModel::MetaLlama38bInstruct)`, `VendorModel::Replicate(ReplicateModel::DeepseekR1)`, etc.

Refer to the specific enums (`OpenAIModel`, `GeminiModel`, `ClaudeModel`, `ReplicateModel`) within the `crates/llm/src/vendors/` subdirectories for all available model variants.

You can provide a `Vec<VendorModel>` to `llm` or `llm_typed` to specify a fallback order.

## Logging

All calls made through `llm_typed` (and potentially `llm` in the future, check implementation details) automatically log the interaction details, including timestamp, prompt, token counts, and outcome (success/error) with details.

*   **Location:** Logs are stored as individual YAML files within the `.ras/prompts/` directory relative to where the application is run.
*   **Format:** Each file contains a serialized `LlmTypedLog` struct.
*   **Purpose:** Useful for debugging prompts, tracking costs (token counts), and analyzing successes/failures.

## Usage Notes

*   **API Keys:** Ensure the necessary environment variables for the respective LLM vendors (e.g., `OPENAI_API_KEY`, `GOOGLE_API_KEY`, `ANTHROPIC_API_KEY`, `REPLICATE_API_TOKEN`) are set.
The crate uses `dotenvy` implicitly in some vendor modules, so a `.env` file might be required depending on the specific vendor implementation details.
*   **Dependencies:** This crate relies on `serde`, `schemars`, `reqwest`, `tokio`, `anyhow`, `tiktoken-rs`, etc. Ensure your project includes `llm` as a dependency in `Cargo.toml`.
*   **Error Handling:** Both `llm` and `llm_typed` return `anyhow::Result`. Handle potential errors appropriately (e.g., network issues, API errors, deserialization failures, validation errors).
*   **Model Choice:** Choose models appropriate for the task. More complex structured output tasks generally require more capable models (like GPT-4o, Claude 3.5 Sonnet, Gemini 1.5 Pro).
