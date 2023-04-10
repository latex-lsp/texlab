pub mod bibtex;
pub mod latex;

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

#[macro_export]
macro_rules! match_ast {
    (match $node:ident { $($tt:tt)* }) => { $crate::match_ast!(match ($node) { $($tt)* }) };

    (match ($node:expr) {
        $( $( $path:ident )::+ ($it:pat) => $res:expr, )*
        _ => $catch_all:expr $(,)?
    }) => {{
        $( if let Some($it) = $($path::)+cast($node.clone()) { $res } else )*
        { $catch_all }
    }};
}
