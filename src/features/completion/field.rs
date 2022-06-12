use lsp_types::CompletionParams;
use rowan::{ast::AstNode, TextRange};

use crate::{
    features::cursor::CursorContext,
    syntax::bibtex::{self, HasName},
    LANGUAGE_DATA,
};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_fields<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
) -> Option<()> {
    let token = context.cursor.as_bibtex()?;

    let range = if token.kind() == bibtex::NAME {
        token.text_range()
    } else {
        TextRange::empty(context.offset)
    };

    let parent = token.parent()?;
    if let Some(entry) = bibtex::Entry::cast(parent.clone()) {
        if entry.name_token()?.text_range() == token.text_range() {
            return None;
        }
    } else {
        bibtex::Field::cast(parent)?;
    }

    for field in &LANGUAGE_DATA.fields {
        let data = InternalCompletionItemData::Field { field };
        let item = InternalCompletionItem::new(range, data);
        items.push(item);
    }
    Some(())
}
