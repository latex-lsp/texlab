use std::path::PathBuf;

use distro::Language;
use syntax::{bibtex, latex, BuildError};
use url::Url;

use crate::{
    diagnostics::{self, Diagnostic},
    semantics,
    util::{LineCol, LineIndex},
    Config,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Owner {
    Client,
    Server,
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
    pub diagnostics: Vec<Diagnostic>,
}

impl Document {
    pub fn parse(
        uri: Url,
        text: String,
        language: Language,
        owner: Owner,
        cursor: LineCol,
        config: &Config,
    ) -> Self {
        let dir = uri.join(".").unwrap();

        let path = if uri.scheme() == "file" {
            uri.to_file_path().ok()
        } else {
            None
        };

        let line_index = LineIndex::new(&text);

        let diagnostics = Vec::new();
        let data = match language {
            Language::Tex => {
                let green = parser::parse_latex(&text, &config.syntax);
                let mut semantics = semantics::tex::Semantics::default();
                semantics.process_root(&latex::SyntaxNode::new_root(green.clone()));
                DocumentData::Tex(TexDocumentData { green, semantics })
            }
            Language::Bib => {
                let green = parser::parse_bibtex(&text);
                DocumentData::Bib(BibDocumentData { green })
            }
            Language::Aux => {
                let green = parser::parse_latex(&text, &config.syntax);
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

        let mut document = Self {
            uri,
            dir,
            path,
            text,
            line_index,
            owner,
            cursor,
            language,
            data,
            diagnostics,
        };

        match language {
            Language::Tex => diagnostics::tex::analyze(&mut document, config),
            Language::Bib => diagnostics::bib::analyze(&mut document),
            Language::Aux | Language::Log | Language::Root | Language::Tectonic => (),
        };

        document
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
