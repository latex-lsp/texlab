use base_db::semantics::Span;
use syntax::bibtex;

use crate::{
    CompletionItem, CompletionItemData, CompletionParams, EntryTypeData, util::CompletionBuilder,
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
        Some(Span::from(&token))
    }
}
