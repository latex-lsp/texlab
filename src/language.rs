use std::{ffi::OsStr, path::Path};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum DocumentLanguage {
    Latex,
    Bibtex,
    BuildLog,
}

impl DocumentLanguage {
    pub fn by_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(OsStr::to_str)
            .and_then(Self::by_extension)
    }

    pub fn by_extension(extension: &str) -> Option<Self> {
        match extension.to_lowercase().as_str() {
            "tex" | "sty" | "cls" | "def" | "lco" | "aux" | "rnw" => Some(Self::Latex),
            "bib" | "bibtex" => Some(Self::Bibtex),
            "log" => Some(Self::BuildLog),
            _ => None,
        }
    }

    pub fn by_language_id(language_id: &str) -> Option<Self> {
        match language_id {
            "latex" | "tex" => Some(Self::Latex),
            "bibtex" | "bib" => Some(Self::Bibtex),
            _ => None,
        }
    }
}
