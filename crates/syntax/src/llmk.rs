use serde::Deserialize;

/// Data structure for `llmk.toml` configuration files.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct LlmkTomlData {
    /// Output directory: points to the directory for aux, log, and pdf files.
    pub output_directory: Option<String>,
}
