use syntax::{bibtex, latex, BuildLog};

use crate::{db::analysis::TexAnalysis, Db};

#[salsa::interned]
pub struct TexDocumentData {
    pub green: rowan::GreenNode,
}

impl TexDocumentData {
    pub fn root(self, db: &dyn Db) -> latex::SyntaxNode {
        latex::SyntaxNode::new_root(self.green(db))
    }
}

#[salsa::tracked]
impl TexDocumentData {
    #[salsa::tracked]
    pub fn analyze(self, db: &dyn Db) -> TexAnalysis {
        let root = latex::SyntaxNode::new_root(self.green(db));
        TexAnalysis::analyze(db, &root)
    }
}

#[salsa::interned]
pub struct BibDocumentData {
    pub green: rowan::GreenNode,
}

impl BibDocumentData {
    pub fn root(self, db: &dyn Db) -> bibtex::SyntaxNode {
        bibtex::SyntaxNode::new_root(self.green(db))
    }
}

#[salsa::interned]
pub struct LogDocumentData {
    pub log: BuildLog,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct TexlabRootData;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct TectonicData;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum DocumentData {
    Tex(TexDocumentData),
    Bib(BibDocumentData),
    Log(LogDocumentData),
    TexlabRoot(TexlabRootData),
    Tectonic(TectonicData),
}

impl DocumentData {
    pub fn as_tex(self) -> Option<TexDocumentData> {
        match self {
            Self::Tex(data) => Some(data),
            _ => None,
        }
    }

    pub fn as_bib(self) -> Option<BibDocumentData> {
        match self {
            Self::Bib(data) => Some(data),
            _ => None,
        }
    }

    pub fn as_log(self) -> Option<LogDocumentData> {
        match self {
            Self::Log(data) => Some(data),
            _ => None,
        }
    }
}
