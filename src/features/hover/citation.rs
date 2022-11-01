use lsp_types::MarkupKind;
use rowan::ast::AstNode;

use crate::{citation, syntax::bibtex, util::cursor::CursorContext};

use super::HoverResult;

pub(super) fn find_citation_hover(context: &CursorContext) -> Option<HoverResult> {
    let (key, range) = context
        .find_citation_key_word()
        .or_else(|| context.find_citation_key_command())
        .or_else(|| context.find_entry_key())?;

    let value = context
        .workspace
        .related(context.db, context.distro, context.document)
        .iter()
        .find_map(|document| {
            let data = document.parse(context.db).as_bib()?;
            let root = data.root(context.db);
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
