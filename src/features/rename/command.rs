use std::sync::Arc;

use lsp_types::RenameParams;
use rowan::{TextRange, TextSize};
use rustc_hash::FxHashMap;

use crate::{
    features::cursor::{CursorContext, HasPosition},
    syntax::latex,
};

use super::{Indel, RenameResult};

pub(super) fn prepare_command_rename<P: HasPosition>(
    context: &CursorContext<P>,
) -> Option<TextRange> {
    context.cursor.command_range(context.offset)
}

pub(super) fn rename_command(context: &CursorContext<RenameParams>) -> Option<RenameResult> {
    prepare_command_rename(context)?;
    let name = context.cursor.as_latex()?.text();
    let mut changes = FxHashMap::default();
    for document in context.request.workspace.documents_by_uri.values() {
        if let Some(data) = document.data.as_latex() {
            let root = latex::SyntaxNode::new_root(data.green.clone());
            let edits = root
                .descendants_with_tokens()
                .filter_map(|element| element.into_token())
                .filter(|token| token.kind().is_command_name() && token.text() == name)
                .map(|token| {
                    let range = token.text_range();
                    Indel {
                        delete: TextRange::new(range.start() + TextSize::from(1), range.end()),
                        insert: context.request.params.new_name.clone(),
                    }
                })
                .collect();

            changes.insert(Arc::clone(&document.uri), edits);
        }
    }

    Some(RenameResult { changes })
}
