use crate::{
    db::analysis::TexAnalysis,
    syntax::{latex, BuildLog},
    Db,
};

#[salsa::interned]
pub struct TexDocumentData {
    pub green: rowan::GreenNode,
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

#[salsa::interned]
pub struct LogDocumentData {
    pub log: BuildLog,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum DocumentData {
    Tex(TexDocumentData),
    Bib(BibDocumentData),
    Log(LogDocumentData),
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
