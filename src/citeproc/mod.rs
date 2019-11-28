mod name;
mod ris;

use crate::syntax::*;
use lsp_types::*;

pub fn render_citation(_tree: &BibtexSyntaxTree, _key: &str) -> Option<MarkupContent> {
    None
}
