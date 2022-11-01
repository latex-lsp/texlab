use rowan::{ast::AstNode, TextRange};

use crate::{
    syntax::bibtex::{self, HasName},
    util::cursor::CursorContext,
    LANGUAGE_DATA,
};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_fields<'db>(
    context: &'db CursorContext,
    items: &mut Vec<InternalCompletionItem<'db>>,
) -> Option<()> {
    let token = context.cursor.as_bib()?;

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
