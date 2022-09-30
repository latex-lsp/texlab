use lsp_types::{HoverParams, MarkupKind};
use rowan::ast::AstNode;

use crate::{
    citation::field::text::TextFieldData,
    features::cursor::CursorContext,
    syntax::bibtex::{self, HasName, HasValue},
};

use super::HoverResult;

pub(super) fn find_string_reference_hover(
    context: &CursorContext<HoverParams>,
) -> Option<HoverResult> {
    let document = context.request.main_document();
    let data = document.data().as_bibtex()?;

    let name = context
        .cursor
        .as_bibtex()
        .filter(|token| token.kind() == bibtex::NAME)
        .filter(|token| {
            let parent = token.parent().unwrap();
            bibtex::Value::can_cast(parent.kind()) || bibtex::StringDef::can_cast(parent.kind())
        })?;

    for string in bibtex::SyntaxNode::new_root(data.green.clone())
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
