use std::{i32, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BibtexFormatter {
    BibtexTidy,
    Latexindent,
}

impl Default for BibtexFormatter {
    fn default() -> Self {
        Self::BibtexTidy
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LineLength(pub i32);

impl LineLength {
    pub fn value(self) -> i32 {
        if self.0 <= 0 {
            i32::MAX
        } else {
            self.0
        }
    }
}

impl Default for LineLength {
    fn default() -> Self {
        Self(120)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BibtexFormattingOptions {
    #[serde(default)]
    pub formatter: BibtexFormatter,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct LatexForwardSearchOptions {
    pub executable: Option<String>,
    pub args: Option<Vec<String>>,
}

fn default_latex_lint_on_change() -> bool {
    false
}

fn default_latex_lint_on_save() -> bool {
    true
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexLintOptions {
    #[serde(default = "default_latex_lint_on_change")]
    pub on_change: bool,

    #[serde(default = "default_latex_lint_on_save")]
    pub on_save: bool,
}

impl Default for LatexLintOptions {
    fn default() -> Self {
        Self {
            on_change: default_latex_lint_on_change(),
            on_save: default_latex_lint_on_save(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexBuildOptions {
    #[serde(default)]
    pub output_directory: Option<PathBuf>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexOptions {
    #[serde(default)]
    pub lint: LatexLintOptions,

    #[serde(default)]
    pub build: LatexBuildOptions,

    #[serde(default)]
    pub forward_search: LatexForwardSearchOptions,

    #[serde(default)]
    pub root_directory: Option<PathBuf>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BibtexOptions {
    #[serde(default)]
    pub formatting: BibtexFormattingOptions,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    #[serde(default)]
    pub latex: LatexOptions,

    #[serde(default)]
    pub bibtex: BibtexOptions,
}
