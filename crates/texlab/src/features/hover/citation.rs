use lsp_types::MarkupKind;
use rowan::ast::AstNode;
use syntax::bibtex;

use crate::util::cursor::CursorContext;

use super::HoverResult;

pub(super) fn find_hover(context: &CursorContext) -> Option<HoverResult> {
    let (key, range) = context
        .find_citation_key_word()
        .or_else(|| context.find_citation_key_command())
        .or_else(|| context.find_entry_key())?;

    let value = context.project.documents.iter().find_map(|document| {
        let data = document.data.as_bib()?;
        let root = bibtex::Root::cast(data.root_node())?;
        let entry = root.find_entry(&key)?;
        citeproc::render(&entry)
    })?;

    Some(HoverResult {
        range,
        value,
        value_kind: MarkupKind::Markdown,
    })
}
