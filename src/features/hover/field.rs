use lsp_types::MarkupKind;
use rowan::ast::AstNode;

use crate::{
    syntax::bibtex,
    util::{cursor::CursorContext, lang_data::LANGUAGE_DATA},
};

use super::HoverResult;

pub(super) fn find_hover(context: &CursorContext) -> Option<HoverResult> {
    let name = context
        .cursor
        .as_bib()
        .filter(|token| token.kind() == bibtex::NAME)?;

    bibtex::Field::cast(name.parent()?)?;

    let docs = LANGUAGE_DATA.field_documentation(name.text())?;
    Some(HoverResult {
        range: name.text_range(),
        value: docs.to_string(),
        value_kind: MarkupKind::Markdown,
    })
}
