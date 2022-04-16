use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    pub root_directory: Option<PathBuf>,

    pub aux_directory: Option<PathBuf>,

    #[serde(default)]
    pub bibtex_formatter: BibtexFormatter,

    #[serde(default)]
    pub latex_formatter: LatexFormatter,

    pub formatter_line_length: Option<i32>,

    pub diagnostics_delay: Option<u64>,

    #[serde(default)]
    pub build: BuildOptions,

    #[serde(default)]
    pub chktex: ChktexOptions,

    #[serde(default)]
    pub latexindent: LatexindentOptions,

    #[serde(default)]
    pub forward_search: ForwardSearchOptions,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BibtexFormatter {
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
pub struct LatexindentOptions {
    pub local: Option<String>,

    #[serde(default)]
    pub modify_line_breaks: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildOptions {
    pub executable: Option<String>,

    pub args: Option<Vec<String>>,

    #[serde(default)]
    pub is_continuous: bool,

    #[serde(default)]
    pub on_save: bool,

    #[serde(default)]
    pub forward_search_after: bool,
}

impl BuildOptions {
    pub fn executable(&self) -> String {
        self.executable
            .as_ref()
            .map(Clone::clone)
            .unwrap_or_else(|| "latexmk".to_string())
    }

    pub fn args(&self) -> Vec<String> {
        self.args.as_ref().map(Clone::clone).unwrap_or_else(|| {
            vec![
                "-pdf".to_string(),
                "-interaction=nonstopmode".to_string(),
                "-synctex=1".to_string(),
                "%f".to_string(),
            ]
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewerOptions {
    #[serde(default)]
    pub enabled: bool,

    pub executable: Option<String>,

    pub args: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChktexOptions {
    #[serde(default)]
    pub on_open_and_save: bool,

    #[serde(default)]
    pub on_edit: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct ForwardSearchOptions {
    pub executable: Option<String>,
    pub args: Option<Vec<String>>,
}
