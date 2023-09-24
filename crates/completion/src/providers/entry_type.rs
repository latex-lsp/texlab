use base_db::semantics::Span;
use rowan::{TextRange, TextSize};
use syntax::bibtex;

use crate::{
    util::CompletionBuilder, CompletionItem, CompletionItemData, CompletionParams, EntryTypeData,
};

pub fn complete_entry_types<'a>(
    params: &'a CompletionParams<'a>,
    builder: &mut CompletionBuilder<'a>,
) -> Option<()> {
    let cursor = find_entry_type(params)?;

    for entry_type in base_db::data::BIBTEX_ENTRY_TYPES {
        if let Some(score) = builder.matcher.score(entry_type.name, &cursor.text) {
            let data = CompletionItemData::EntryType(EntryTypeData(*entry_type));
            builder
                .items
                .push(CompletionItem::new_simple(score, cursor.range, data));
        }
    }

    Some(())
}

fn find_entry_type(params: &CompletionParams) -> Option<Span> {
    let data = params.feature.document.data.as_bib()?;

    let token = data
        .root_node()
        .token_at_offset(params.offset)
        .find(|token| token.kind() == bibtex::TYPE)?;

    let range = token.text_range();
    if range.start() == params.offset {
        None
    } else {
        let text = &token.text()[1..];
        let range = TextRange::new(range.start() + TextSize::of('@'), range.end());
        Some(Span::new(text.into(), range))
    }
}
