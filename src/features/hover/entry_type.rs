use lsp_types::MarkupKind;

use crate::{syntax::bibtex, util::cursor::CursorContext, LANGUAGE_DATA};

use super::HoverResult;

pub(super) fn find_entry_type_hover(context: &CursorContext) -> Option<HoverResult> {
    let name = context
        .cursor
        .as_bib()
        .filter(|token| token.kind() == bibtex::TYPE)?;

    let docs = LANGUAGE_DATA.entry_type_documentation(&name.text()[1..])?;
    Some(HoverResult {
        range: name.text_range(),
        value: docs.to_string(),
        value_kind: MarkupKind::Markdown,
    })
}
