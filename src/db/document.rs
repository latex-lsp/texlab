use rowan::GreenNode;

use crate::{
    parser::{parse_bibtex, parse_build_log, parse_latex},
    syntax::{latex, BuildError, BuildLog},
    Db, LineIndex,
};

use super::{FileId, Language};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum OpenedBy {
    Client,
    Server,
}

#[salsa::input]
pub struct Document {
    pub file: FileId,
    #[return_ref]
    pub text: String,
    pub language: Language,
    pub opened_by: OpenedBy,
}

#[salsa::tracked]
impl Document {
    #[salsa::tracked]
    pub fn parse(self, db: &dyn Db) -> DocumentData {
        let text = self.text(db);
        match self.language(db) {
            Language::Tex => {
                let green = parse_latex(text);
                DocumentData::Tex(TexDocumentData::new(db, green))
            }
            Language::Bib => {
                let green = parse_bibtex(text);
                DocumentData::Bib(BibDocumentData::new(db, green))
            }
            Language::Log => {
                let BuildLog { errors } = parse_build_log(text);
                DocumentData::Log(LogDocumentData::new(db, errors))
            }
        }
    }

    #[salsa::tracked(return_ref)]
    pub fn line_index(self, db: &dyn Db) -> LineIndex {
        LineIndex::new(self.text(db))
    }

    #[salsa::tracked]
    pub fn can_be_root(self, db: &dyn Db) -> bool {
        match self.parse(db) {
            DocumentData::Tex(data) => data.extras(db).can_be_root,
            DocumentData::Bib(_) | DocumentData::Log(_) => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum DocumentData {
    Tex(TexDocumentData),
    Bib(BibDocumentData),
    Log(LogDocumentData),
}

#[salsa::tracked]
pub struct TexDocumentData {
    pub green: GreenNode,
}

#[salsa::tracked]
impl TexDocumentData {
    #[salsa::tracked(return_ref)]
    pub fn extras(self, db: &dyn Db) -> latex::Extras {
        let extras = latex::Extras::default();
        latex::SyntaxNode::new_root(self.green(db));
        extras
    }
}

#[salsa::tracked]
pub struct BibDocumentData {
    pub green: GreenNode,
}

#[salsa::tracked]
pub struct LogDocumentData {
    #[return_ref]
    pub errors: Vec<BuildError>,
}
