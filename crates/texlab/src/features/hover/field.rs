use base_db::data::BibtexFieldType;
use lsp_types::MarkupKind;
use rowan::ast::AstNode;
use syntax::bibtex;

use crate::util::cursor::CursorContext;

use super::HoverResult;

pub(super) fn find_hover(context: &CursorContext) -> Option<HoverResult> {
    let name = context
        .cursor
        .as_bib()
        .filter(|token| token.kind() == bibtex::NAME)?;

    bibtex::Field::cast(name.parent()?)?;

    let docs = BibtexFieldType::find(name.text())?.documentation;
    Some(HoverResult {
        range: name.text_range(),
        value: docs.into(),
        value_kind: MarkupKind::Markdown,
    })
}
