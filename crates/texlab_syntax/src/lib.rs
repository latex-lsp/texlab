mod bibtex;
mod language;
mod latex;
mod lsp_kind;
mod text;

pub use self::bibtex::*;
pub use self::language::*;
pub use self::latex::*;
pub use self::lsp_kind::*;
pub use self::text::*;

use std::path::PathBuf;
use texlab_distro::{Language, Resolver};
use texlab_protocol::{Options, Uri};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SyntaxTreeInput<'a> {
    pub options: &'a Options,
    pub resolver: &'a Resolver,
    pub uri: &'a Uri,
    pub text: &'a str,
    pub language: Language,
}

impl<'a> SyntaxTreeInput<'a> {
    pub fn base_path(&self) -> Option<PathBuf> {
        self.options
            .latex
            .as_ref()
            .and_then(|opts| opts.root_directory.as_ref())
            .and_then(|path| dunce::canonicalize(path).ok())
            .or_else(|| {
                self.uri.to_file_path().ok().map(|mut path| {
                    path.pop();
                    path
                })
            })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SyntaxTree {
    Latex(Box<LatexSyntaxTree>),
    Bibtex(Box<BibtexSyntaxTree>),
}

impl SyntaxTree {
    pub fn parse(input: SyntaxTreeInput) -> Self {
        match input.language {
            Language::Latex => SyntaxTree::Latex(Box::new(LatexSyntaxTree::parse(input))),
            Language::Bibtex => SyntaxTree::Bibtex(Box::new(input.text.into())),
        }
    }
}
