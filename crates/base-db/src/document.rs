use std::path::PathBuf;

use distro::Language;
use rowan::TextSize;
use syntax::{latex, BuildError};
use url::Url;

use crate::{line_index::LineIndex, semantics};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Owner {
    Client,
    Server,
}

#[derive(Debug)]
pub struct Document {
    pub uri: Url,
    pub dir: Url,
    pub path: Option<PathBuf>,
    pub text: String,
    pub line_index: LineIndex,
    pub owner: Owner,
    pub cursor: TextSize,
    pub chktex: Vec<()>,
    pub language: Language,
    pub data: DocumentData,
}

impl Document {
    pub fn parse(uri: Url, text: String, language: Language, owner: Owner) -> Self {
        let dir = uri.join(".").unwrap();

        let path = if uri.scheme() == "file" {
            uri.to_file_path().ok()
        } else {
            None
        };

        let line_index = LineIndex::new(&text);

        let cursor = TextSize::from(0);
        let chktex = Vec::new();
        let data = match language {
            Language::Tex => {
                let green = parser::parse_latex(&text);
                let mut semantics = semantics::tex::Semantics::default();
                semantics.process_root(&latex::SyntaxNode::new_root(green.clone()));
                DocumentData::Tex(TexDocumentData { green, semantics })
            }
            Language::Bib => {
                let green = parser::parse_bibtex(&text);
                DocumentData::Bib(BibDocumentData { green })
            }
            Language::Log => {
                let errors = parser::parse_build_log(&text).errors;
                DocumentData::Log(LogDocumentData { errors })
            }
            Language::Root => DocumentData::Root,
            Language::Tectonic => DocumentData::Tectonic,
        };

        Self {
            uri,
            dir,
            path,
            text,
            line_index,
            owner,
            cursor,
            chktex,
            language,
            data,
        }
    }
}

impl std::borrow::Borrow<Url> for Document {
    fn borrow(&self) -> &Url {
        &self.uri
    }
}

impl std::borrow::Borrow<str> for Document {
    fn borrow(&self) -> &str {
        self.uri.as_str()
    }
}

impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri
    }
}

impl Eq for Document {}

impl std::hash::Hash for Document {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uri.hash(state)
    }
}

#[derive(Debug)]
pub enum DocumentData {
    Tex(TexDocumentData),
    Bib(BibDocumentData),
    Aux(AuxDocumentData),
    Log(LogDocumentData),
    Root,
    Tectonic,
}

#[derive(Debug)]
pub struct TexDocumentData {
    pub green: rowan::GreenNode,
    pub semantics: semantics::tex::Semantics,
}

#[derive(Debug)]
pub struct BibDocumentData {
    pub green: rowan::GreenNode,
}

#[derive(Debug)]
pub struct LogDocumentData {
    pub errors: Vec<BuildError>,
}

#[derive(Debug)]
pub struct AuxDocumentData {
    pub green: rowan::GreenNode,
    pub semantics: semantics::aux::Semantics,
}
