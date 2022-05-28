use lsp_types::{HoverParams, MarkupKind};
use rowan::ast::AstNode;

use crate::{
    features::cursor::CursorContext,
    syntax::bibtex::{self, HasName, HasValue},
};

use super::HoverResult;

pub(super) fn find_string_reference_hover(
    context: &CursorContext<HoverParams>,
) -> Option<HoverResult> {
    let data = context.request.main_document().data.as_bibtex()?;

    let key = context
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
            .filter(|k| k.text() == key.text())
            .is_some()
        {
            let value = string.value()?.syntax().text().to_string();
            return Some(HoverResult {
                range: key.text_range(),
                value,
                value_kind: MarkupKind::PlainText,
            });
        }
    }

    None
}
