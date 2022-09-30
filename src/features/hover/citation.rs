use lsp_types::{HoverParams, MarkupKind};
use rowan::ast::AstNode;

use crate::{citation, features::cursor::CursorContext, syntax::bibtex};

use super::HoverResult;

pub(super) fn find_citation_hover(context: &CursorContext<HoverParams>) -> Option<HoverResult> {
    let (key, range) = context
        .find_citation_key_word()
        .or_else(|| context.find_citation_key_command())
        .or_else(|| context.find_entry_key())?;

    let value = context.request.workspace.iter().find_map(|document| {
        let data = document.data().as_bibtex()?;
        let root = bibtex::SyntaxNode::new_root(data.green.clone());
        let root = bibtex::Root::cast(root)?;
        let entry = root.find_entry(&key)?;
        citation::render(&entry)
    })?;

    Some(HoverResult {
        range,
        value,
        value_kind: MarkupKind::Markdown,
    })
}
