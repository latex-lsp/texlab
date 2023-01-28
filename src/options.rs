use std::{path::PathBuf, time::Duration};

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Options {
    pub root_directory: Option<PathBuf>,
    pub aux_directory: Option<PathBuf>,
    pub bibtex_formatter: BibtexFormatter,
    pub latex_formatter: LatexFormatter,
    pub formatter_line_length: Option<i32>,
    pub diagnostics: DiagnosticsOptions,
    pub diagnostics_delay: DiagnosticsDelay,
    pub build: BuildOptions,
    pub chktex: ChktexOptions,
    pub latexindent: LatexindentOptions,
    pub forward_search: ForwardSearchOptions,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct DiagnosticsDelay(#[serde(with = "serde_millis")] pub Duration);

impl Default for DiagnosticsDelay {
    fn default() -> Self {
        Self(Duration::from_millis(300))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BibtexFormatter {
    None,
    Texlab,
    Latexindent,
}

impl Default for BibtexFormatter {
    fn default() -> Self {
        Self::Texlab
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LatexFormatter {
    None,
    Texlab,
    Latexindent,
}

impl Default for LatexFormatter {
    fn default() -> Self {
        Self::Latexindent
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct LatexindentOptions {
    pub local: Option<String>,
    pub modify_line_breaks: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct BuildOptions {
    pub executable: BuildExecutable,
    pub args: BuildArgs,
    pub on_save: bool,
    pub forward_search_after: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BuildExecutable(pub String);

impl Default for BuildExecutable {
    fn default() -> Self {
        Self("latexmk".to_string())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BuildArgs(pub Vec<String>);

impl Default for BuildArgs {
    fn default() -> Self {
        Self(vec![
            "-pdf".to_string(),
            "-interaction=nonstopmode".to_string(),
            "-synctex=1".to_string(),
            "%f".to_string(),
        ])
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct ChktexOptions {
    pub on_open_and_save: bool,
    pub on_edit: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct ForwardSearchOptions {
    pub executable: Option<String>,
    pub args: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct DiagnosticsOptions {
    pub allowed_patterns: Vec<DiagnosticsPattern>,
    pub ignored_patterns: Vec<DiagnosticsPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsPattern(#[serde(with = "serde_regex")] pub Regex);

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct StartupOptions {
    pub skip_distro: bool,
}
