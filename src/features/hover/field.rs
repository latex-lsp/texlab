use lsp_types::{HoverParams, MarkupKind};
use rowan::ast::AstNode;

use crate::{features::cursor::CursorContext, syntax::bibtex, LANGUAGE_DATA};

use super::HoverResult;

pub(super) fn find_field_hover(context: &CursorContext<HoverParams>) -> Option<HoverResult> {
    let name = context
        .cursor
        .as_bibtex()
        .filter(|token| token.kind() == bibtex::NAME)?;

    bibtex::Field::cast(name.parent()?)?;

    let docs = LANGUAGE_DATA.field_documentation(name.text())?;
    Some(HoverResult {
        range: name.text_range(),
        value: docs.to_string(),
        value_kind: MarkupKind::Markdown,
    })
}
