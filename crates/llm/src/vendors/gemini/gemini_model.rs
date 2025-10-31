//!
//! Defines the `GeminiModel` enum and its associated methods.
//!
//! This enum represents the available Google Gemini models supported by the application.
//! It provides methods to get the model ID string, aliases, and maximum token count.
//! Adheres to the one-item-per-file guideline.
//! Extracted from `completion.rs`.

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum GeminiModel {
    #[default]
    Gemini20ProExp0205,
    Gemini20Flash,
    Gemini20FlashLite,
    Gemini20FlashThinkingExp0121,
    Gemini25ProPreview0325,
    Gemini25Pro,
    Gemini25Flash,
    Gemini25FlashImage,
}

impl std::fmt::Display for GeminiModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id())
    }
}

impl GeminiModel {
    pub fn id(&self) -> &'static str {
        match self {
            GeminiModel::Gemini25ProPreview0325 => "gemini-2.5-pro-preview-03-25",
            GeminiModel::Gemini20ProExp0205 => "gemini-2.0-pro-exp-02-05",
            GeminiModel::Gemini20Flash => "gemini-2.0-flash",
            GeminiModel::Gemini20FlashLite => "gemini-2.0-flash-lite",
            GeminiModel::Gemini20FlashThinkingExp0121 => "gemini-2.0-flash-thinking-exp-01-21",
            GeminiModel::Gemini25Pro => "gemini-2.5-pro",
            GeminiModel::Gemini25Flash => "gemini-2.5-flash",
            GeminiModel::Gemini25FlashImage => "gemini-2.5-flash-image-preview",
        }
    }

    pub fn aliases(&self) -> std::vec::Vec<&'static str> {
        match self {
            GeminiModel::Gemini25ProPreview0325 => vec!["gemini-2.5-pro", "2.5", "2.5-pro"],
            GeminiModel::Gemini20ProExp0205 => vec!["gemini-2.0-pro", "2.0-pro"],
            GeminiModel::Gemini20Flash => vec!["gemini-flash", "flash"],
            GeminiModel::Gemini20FlashLite => vec!["gemini-flash-lite", "flash-lite"],
            GeminiModel::Gemini20FlashThinkingExp0121 => vec!["gemini-flash-thinking", "flash-thinking"],
            GeminiModel::Gemini25Pro => vec![],
            GeminiModel::Gemini25Flash => vec![],
            GeminiModel::Gemini25FlashImage => vec!["nano-banana", "gemini-flash-image"]
        }
    }

    pub fn max_tokens(&self) -> u32 {
        match self {
            GeminiModel::Gemini20ProExp0205 => 8192,
            GeminiModel::Gemini20Flash => 8192,
            GeminiModel::Gemini20FlashLite => 8192,
            GeminiModel::Gemini20FlashThinkingExp0121 => 8192,
            GeminiModel::Gemini25ProPreview0325 => 65536,
            GeminiModel::Gemini25Pro => 65536,
            GeminiModel::Gemini25Flash => 65536,
            GeminiModel::Gemini25FlashImage => 65536
        }
    }
}
