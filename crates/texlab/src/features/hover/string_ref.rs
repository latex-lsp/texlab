use lsp_types::MarkupKind;
use rowan::ast::AstNode;
use syntax::bibtex::{self, HasName, HasValue};

use crate::{citation::field::text::TextFieldData, util::cursor::CursorContext};

use super::HoverResult;

pub(super) fn find_hover(context: &CursorContext) -> Option<HoverResult> {
    let data = context.document.parse(context.db).as_bib()?;

    let name = context
        .cursor
        .as_bib()
        .filter(|token| token.kind() == bibtex::NAME)
        .filter(|token| {
            let parent = token.parent().unwrap();
            bibtex::Value::can_cast(parent.kind()) || bibtex::StringDef::can_cast(parent.kind())
        })?;

    for string in data
        .root(context.db)
        .children()
        .filter_map(bibtex::StringDef::cast)
    {
        if string
            .name_token()
            .map_or(false, |token| token.text() == name.text())
        {
            let value = TextFieldData::parse(&string.value()?)?.text;
            return Some(HoverResult {
                range: name.text_range(),
                value,
                value_kind: MarkupKind::PlainText,
            });
        }
    }

    None
}
