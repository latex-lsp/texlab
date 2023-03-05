use rowan::{TextRange, TextSize};
use rustc_hash::FxHashMap;

use crate::{syntax::latex, util::cursor::CursorContext};

use super::{Indel, Params, RenameResult};

pub(super) fn prepare_rename<T>(context: &CursorContext<T>) -> Option<TextRange> {
    context.cursor.command_range(context.offset)
}

pub(super) fn rename(context: &CursorContext<Params>) -> Option<RenameResult> {
    prepare_rename(context)?;
    let name = context.cursor.as_tex()?.text();
    let mut changes = FxHashMap::default();
    for document in context.related() {
        if let Some(data) = document.parse(context.db).as_tex() {
            let root = data.root(context.db);
            let edits = root
                .descendants_with_tokens()
                .filter_map(|element| element.into_token())
                .filter(|token| token.kind() == latex::COMMAND_NAME && token.text() == name)
                .map(|token| {
                    let range = token.text_range();
                    Indel {
                        delete: TextRange::new(range.start() + TextSize::from(1), range.end()),
                        insert: context.params.new_name.clone(),
                    }
                })
                .collect();

            changes.insert(document, edits);
        }
    }

    Some(RenameResult { changes })
}
