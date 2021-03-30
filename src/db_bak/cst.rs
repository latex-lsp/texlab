use std::fmt;

use cstree::GreenNode;

use crate::{bibtex, latex, DocumentLanguage};

use super::{Document, DocumentDatabase};

#[derive(PartialEq, Eq, Clone, Hash)]
pub enum Cst {
    Latex(latex::SyntaxNode),
    Bibtex(bibtex::SyntaxNode),
    BuildLog,
}

impl fmt::Debug for Cst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CST")
    }
}

#[salsa::query_group(CstDatabaseStorage)]
pub trait CstDatabase: DocumentDatabase {
    fn cst(&self, document: Document) -> Cst;
}

fn cst(db: &dyn CstDatabase, document: Document) -> Cst {
    let source_code = db.source_code(document);
    match db.source_language(document) {
        DocumentLanguage::Latex => Cst::Latex(latex::SyntaxNode::new_root(
            latex::parse(&source_code).green_node,
        )),
        DocumentLanguage::Bibtex => Cst::Bibtex(bibtex::SyntaxNode::new_root(
            bibtex::parse(&source_code).green_node,
        )),
        DocumentLanguage::BuildLog => Cst::BuildLog,
    }
}
