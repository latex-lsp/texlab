use base_db::{Document, DocumentData};
use rowan::TextRange;
use rustc_hash::FxHashMap;

use crate::util::cursor::CursorContext;

use super::{Indel, Params, RenameResult};

pub(super) fn prepare_rename<T>(context: &CursorContext<T>) -> Option<TextRange> {
    let (_, range) = context
        .find_citation_key_word()
        .or_else(|| context.find_entry_key())?;

    Some(range)
}

pub(super) fn rename<'a>(context: &CursorContext<'a, Params>) -> Option<RenameResult<'a>> {
    prepare_rename(context)?;
    let (key_text, _) = context
        .find_citation_key_word()
        .or_else(|| context.find_entry_key())?;

    let mut changes: FxHashMap<&Document, Vec<Indel>> = FxHashMap::default();
    for document in &context.project.documents {
        if let DocumentData::Tex(data) = &document.data {
            let edits = data
                .semantics
                .citations
                .iter()
                .filter(|citation| citation.name.text == key_text)
                .map(|citation| Indel {
                    delete: citation.name.range,
                    insert: context.params.new_name.clone(),
                })
                .collect();

            changes.insert(document, edits);
        } else if let DocumentData::Bib(data) = &document.data {
            let edits = data
                .semantics
                .entries
                .iter()
                .filter(|entry| entry.name.text == key_text)
                .map(|entry| Indel {
                    delete: entry.name.range,
                    insert: context.params.new_name.clone(),
                })
                .collect();

            changes.insert(document, edits);
        }
    }

    Some(RenameResult { changes })
}
