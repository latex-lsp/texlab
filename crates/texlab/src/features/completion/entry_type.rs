use rowan::{TextRange, TextSize};
use syntax::bibtex;

use crate::util::cursor::CursorContext;

use super::builder::CompletionBuilder;

pub fn complete<'db>(
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

    for entry_type in base_db::data::BIBTEX_ENTRY_TYPES {
        builder.entry_type(range, entry_type);
    }

    Some(())
}
