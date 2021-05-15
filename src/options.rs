use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    pub root_directory: Option<PathBuf>,

    pub aux_directory: Option<PathBuf>,

    #[serde(default)]
    pub bibtex_formatter: BibtexFormatter,

    pub diagnostics_delay: Option<u64>,

    #[serde(default)]
    pub build: BuildOptions,

    #[serde(default)]
    pub chktex: ChktexOptions,

    pub forward_search: Option<ForwardSearchOptions>,
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

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildOptions {
    pub executable: Option<String>,

    pub args: Option<Vec<String>>,

    #[serde(default)]
    pub is_continuous: bool,
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
