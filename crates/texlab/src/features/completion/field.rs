use rowan::{ast::AstNode, TextRange};
use syntax::bibtex::{self, HasName};

use crate::util::cursor::CursorContext;

use super::builder::CompletionBuilder;

pub fn complete<'db>(
    context: &'db CursorContext,
    builder: &mut CompletionBuilder<'db>,
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

    for field in base_db::data::BIBTEX_FIELD_TYPES {
        builder.field(range, field);
    }

    Some(())
}
