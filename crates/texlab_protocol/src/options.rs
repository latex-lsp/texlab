use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BibtexFormattingOptions {
    pub line_length: Option<i32>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct LatexForwardSearchOptions {
    pub executable: Option<String>,
    pub args: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LatexLintOptions {
    pub on_change: Option<bool>,
    pub on_save: Option<bool>,
}

impl LatexLintOptions {
    pub fn on_change(&self) -> bool {
        self.on_change.unwrap_or(false)
    }

    pub fn on_save(&self) -> bool {
        self.on_save.unwrap_or(false)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexBuildOptions {
    pub executable: Option<String>,
    pub args: Option<Vec<String>>,
    pub on_save: Option<bool>,
    pub output_directory: Option<PathBuf>,
}

impl LatexBuildOptions {
    pub fn executable(&self) -> String {
        self.executable
            .as_ref()
            .map(Clone::clone)
            .unwrap_or_else(|| "latexmk".to_owned())
    }

    pub fn args(&self) -> Vec<String> {
        self.args.as_ref().map(Clone::clone).unwrap_or_else(|| {
            vec![
                "-pdf".to_owned(),
                "-interaction=nonstopmode".to_owned(),
                "-synctex=1".to_owned(),
            ]
        })
    }

    pub fn on_save(&self) -> bool {
        self.on_save.unwrap_or(false)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexOptions {
    pub forward_search: Option<LatexForwardSearchOptions>,
    pub lint: Option<LatexLintOptions>,
    pub build: Option<LatexBuildOptions>,
    pub root_directory: Option<PathBuf>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BibtexOptions {
    pub formatting: Option<BibtexFormattingOptions>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    pub latex: Option<LatexOptions>,
    pub bibtex: Option<BibtexOptions>,
}

impl Options {
    pub fn resolve_output_file(&self, tex_path: &Path, extension: &str) -> Option<PathBuf> {
        let stem = tex_path.file_stem()?.to_str()?;
        let name = format!("{}.{}", stem, extension);

        self.latex
            .as_ref()
            .and_then(|latex| latex.build.as_ref())
            .and_then(|build| build.output_directory.as_ref())
            .map(|path| path.join(&name))
            .and_then(|path| dunce::canonicalize(path).ok())
            .or_else(|| {
                self.latex
                    .as_ref()
                    .and_then(|latex| latex.root_directory.as_ref())
                    .map(|path| path.join(&name))
                    .and_then(|path| dunce::canonicalize(path).ok())
            })
            .or_else(|| {
                tex_path
                    .parent()
                    .map(|path| path.join(&name))
                    .and_then(|path| dunce::canonicalize(path).ok())
            })
    }
}
