use crate::syntax::latex::ast::*;
use lsp_types::Uri;
use path_clean::PathClean;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LatexIncludeKind {
    TexFile,
    BibFile,
    Package,
    Class,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexInclude {
    pub command: Arc<LatexCommand>,
    target: Option<Uri>,
}

impl LatexInclude {
    fn parse(uri: &Uri, command: Arc<LatexCommand>) -> Option<Self> {
        let mut include = LatexInclude {
            command,
            target: None,
        };

        let mut path = uri.to_file_path().ok()?;
        path.pop();
        path.push(include.path().text());
        path = path.clean();
        let has_extension = path.extension().is_some();
        let mut path = path.to_str()?.to_owned();
        if !has_extension {
            path = format!("{}{}", path, include.extension());
        }
        include.target = Some(Uri::from_file_path(path).ok()?);
        Some(include)
    }

    pub fn parse_all(uri: &Uri, commands: &[Arc<LatexCommand>]) -> Vec<Self> {
        let mut includes = Vec::new();
        for command in commands {
            if INCLUDE_COMMANDS.contains(&command.name.text()) && command.has_word(0) {
                if let Some(include) = LatexInclude::parse(uri, Arc::clone(&command)) {
                    includes.push(include);
                }
            }
        }
        includes
    }

    pub fn target(&self) -> &Uri {
        self.target.as_ref().unwrap()
    }

    pub fn path(&self) -> &LatexToken {
        self.command.extract_word(0).unwrap()
    }

    pub fn kind(&self) -> LatexIncludeKind {
        match self.command.name.text() {
            "\\include" | "\\input" => LatexIncludeKind::TexFile,
            "\\bibliography" | "\\addbibresource" => LatexIncludeKind::BibFile,
            "\\usepackage" => LatexIncludeKind::Package,
            "\\documentclass" => LatexIncludeKind::Class,
            _ => unreachable!(),
        }
    }

    pub fn name(&self) -> Option<String> {
        match self.kind() {
            LatexIncludeKind::TexFile | LatexIncludeKind::BibFile => None,
            LatexIncludeKind::Package => Some(format!("{}.sty", self.path().text())),
            LatexIncludeKind::Class => Some(format!("{}.cls", self.path().text())),
        }
    }

    pub fn extension(&self) -> &'static str {
        match self.kind() {
            LatexIncludeKind::TexFile => ".tex",
            LatexIncludeKind::BibFile => ".bib",
            LatexIncludeKind::Package => ".sty",
            LatexIncludeKind::Class => ".cls",
        }
    }
}

pub static INCLUDE_COMMANDS: &[&str] = &[
    "\\include",
    "\\input",
    "\\bibliography",
    "\\addbibresource",
    "\\usepackage",
    "\\documentclass",
];
