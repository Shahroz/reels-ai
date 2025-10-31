//! Defines the type of tool that can be provided to the OpenAI model.
//!
//! Currently, only 'function' tools are typically supported for function calling.
//! Used within the Tool and ToolChoice structures.
//! Specifies the nature of the available tools.
//! Serialization uses lowercase names.

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    Function,
}
