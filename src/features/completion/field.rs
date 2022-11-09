use rowan::{ast::AstNode, TextRange};

use crate::{
    syntax::bibtex::{self, HasName},
    util::{cursor::CursorContext, lang_data::LANGUAGE_DATA},
};

use super::builder::CompletionBuilder;

pub fn complete_fields<'db>(
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

    for field in &LANGUAGE_DATA.fields {
        builder.field(range, field);
    }

    Some(())
}
