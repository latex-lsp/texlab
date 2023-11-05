use std::path::PathBuf;

use distro::Language;
use line_index::{LineCol, LineIndex};
use rowan::TextRange;
use syntax::{bibtex, latex, BuildError};
use url::Url;

use crate::{semantics, Config};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Owner {
    Client,
    Server,
}

#[derive(Debug)]
pub struct DocumentParams<'a> {
    pub uri: Url,
    pub text: String,
    pub language: Language,
    pub owner: Owner,
    pub cursor: LineCol,
    pub config: &'a Config,
}

#[derive(Clone)]
pub struct Document {
    pub uri: Url,
    pub dir: Url,
    pub path: Option<PathBuf>,
    pub text: String,
    pub line_index: LineIndex,
    pub owner: Owner,
    pub cursor: LineCol,
    pub language: Language,
    pub data: DocumentData,
}

impl Document {
    pub fn parse(params: DocumentParams) -> Self {
        let DocumentParams { uri, text, .. } = params;

        let dir = uri.join(".").unwrap();

        let path = if uri.scheme() == "file" {
            uri.to_file_path().ok()
        } else {
            None
        };

        let line_index = LineIndex::new(&text);

        let data = match params.language {
            Language::Tex => {
                let green = parser::parse_latex(&text, &params.config.syntax);
                let mut semantics = semantics::tex::Semantics::default();
                semantics.process_root(&latex::SyntaxNode::new_root(green.clone()));
                DocumentData::Tex(TexDocumentData { green, semantics })
            }
            Language::Bib => {
                let green = parser::parse_bibtex(&text);
                let mut semantics = semantics::bib::Semantics::default();
                semantics.process_root(&bibtex::SyntaxNode::new_root(green.clone()));
                DocumentData::Bib(BibDocumentData { green, semantics })
            }
            Language::Aux => {
                let green = parser::parse_latex(&text, &params.config.syntax);
                let mut semantics = semantics::auxiliary::Semantics::default();
                semantics.process_root(&latex::SyntaxNode::new_root(green.clone()));
                DocumentData::Aux(AuxDocumentData { green, semantics })
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
            owner: params.owner,
            cursor: params.cursor,
            language: params.language,
            data,
        }
    }
}

impl std::fmt::Debug for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Document").field(&self.uri.as_str()).finish()
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

#[derive(Debug, Clone)]
pub enum DocumentData {
    Tex(TexDocumentData),
    Bib(BibDocumentData),
    Aux(AuxDocumentData),
    Log(LogDocumentData),
    Root,
    Tectonic,
}

impl DocumentData {
    pub fn as_tex(&self) -> Option<&TexDocumentData> {
        if let DocumentData::Tex(data) = self {
            Some(data)
        } else {
            None
        }
    }

    pub fn as_bib(&self) -> Option<&BibDocumentData> {
        if let DocumentData::Bib(data) = self {
            Some(data)
        } else {
            None
        }
    }

    pub fn as_aux(&self) -> Option<&AuxDocumentData> {
        if let DocumentData::Aux(data) = self {
            Some(data)
        } else {
            None
        }
    }

    pub fn as_log(&self) -> Option<&LogDocumentData> {
        if let DocumentData::Log(data) = self {
            Some(data)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct TexDocumentData {
    pub green: rowan::GreenNode,
    pub semantics: semantics::tex::Semantics,
}

impl TexDocumentData {
    pub fn root_node(&self) -> latex::SyntaxNode {
        latex::SyntaxNode::new_root(self.green.clone())
    }
}

#[derive(Debug, Clone)]
pub struct BibDocumentData {
    pub green: rowan::GreenNode,
    pub semantics: semantics::bib::Semantics,
}

impl BibDocumentData {
    pub fn root_node(&self) -> bibtex::SyntaxNode {
        bibtex::SyntaxNode::new_root(self.green.clone())
    }
}

#[derive(Debug, Clone)]
pub struct LogDocumentData {
    pub errors: Vec<BuildError>,
}

#[derive(Debug, Clone)]
pub struct AuxDocumentData {
    pub green: rowan::GreenNode,
    pub semantics: semantics::auxiliary::Semantics,
}

#[derive(Debug, Clone)]
pub struct DocumentLocation<'a> {
    pub document: &'a Document,
    pub range: TextRange,
}

impl<'a> DocumentLocation<'a> {
    pub fn new(document: &'a Document, range: TextRange) -> Self {
        Self { document, range }
    }
}
