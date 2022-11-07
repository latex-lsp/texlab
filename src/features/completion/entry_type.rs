use rowan::{TextRange, TextSize};

use crate::{syntax::bibtex, util::cursor::CursorContext, LANGUAGE_DATA};

use super::builder::CompletionBuilder;

pub fn complete_entry_types<'db>(
    context: &'db CursorContext,
    builder: &mut CompletionBuilder<'db>,
) -> Option<()> {
    let range = context
        .cursor
        .as_bib()
        .filter(|token| token.kind() == bibtex::TYPE)
        .map(bibtex::SyntaxToken::text_range)
        .filter(|range| range.start() != context.offset)
        .map(|range| TextRange::new(range.start() + TextSize::from(1), range.end()))?;

    for entry_type in &LANGUAGE_DATA.entry_types {
        builder.entry_type(range, entry_type);
    }

    Some(())
}
