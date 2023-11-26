pub mod bibtex;
pub mod latex;
pub mod latexmkrc;

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum BuildErrorLevel {
    Error,
    Warning,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct BuildError {
    pub relative_path: std::path::PathBuf,
    pub level: BuildErrorLevel,
    pub message: String,
    pub hint: Option<String>,
    pub line: Option<u32>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct BuildLog {
    pub errors: Vec<BuildError>,
}
