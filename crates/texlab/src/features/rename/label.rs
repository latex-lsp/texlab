use base_db::DocumentData;
use rowan::TextRange;
use rustc_hash::FxHashMap;

use crate::util::cursor::CursorContext;

use super::{Indel, Params, RenameResult};

pub(super) fn prepare_rename<T>(context: &CursorContext<T>) -> Option<TextRange> {
    let (_, range) = context.find_label_name_key()?;
    Some(range)
}

pub(super) fn rename<'a>(context: &CursorContext<'a, Params>) -> Option<RenameResult<'a>> {
    prepare_rename(context)?;
    let (name_text, _) = context.find_label_name_key()?;

    let mut changes = FxHashMap::default();
    for document in &context.related {
        let DocumentData::Tex(data) = &document.data else { continue };

        let edits = data
            .semantics
            .labels
            .iter()
            .filter(|label| label.name.text == name_text)
            .map(|label| Indel {
                delete: label.name.range,
                insert: context.params.new_name.clone(),
            })
            .collect();

        changes.insert(*document, edits);
    }

    Some(RenameResult { changes })
}
