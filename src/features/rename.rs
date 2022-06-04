mod command;
mod entry;
mod label;

use std::sync::Arc;

use lsp_types::{Range, RenameParams, TextDocumentPositionParams, TextEdit, Url, WorkspaceEdit};
use rowan::TextRange;
use rustc_hash::FxHashMap;

use crate::LineIndexExt;

use self::{
    command::{prepare_command_rename, rename_command},
    entry::{prepare_entry_rename, rename_entry},
    label::{prepare_label_rename, rename_label},
};

use super::{cursor::CursorContext, FeatureRequest};

pub fn prepare_rename_all(request: FeatureRequest<TextDocumentPositionParams>) -> Option<Range> {
    let context = CursorContext::new(request);
    let range = prepare_entry_rename(&context)
        .or_else(|| prepare_label_rename(&context))
        .or_else(|| prepare_command_rename(&context))?;

    let line_index = &context.request.main_document().line_index;
    Some(line_index.line_col_lsp_range(range))
}

pub fn rename_all(request: FeatureRequest<RenameParams>) -> Option<WorkspaceEdit> {
    let context = CursorContext::new(request);
    let result = rename_entry(&context)
        .or_else(|| rename_label(&context))
        .or_else(|| rename_command(&context))?;

    let changes = result
        .changes
        .into_iter()
        .map(|(uri, old_edits)| {
            let document = &context.request.workspace.documents_by_uri[&uri];
            let new_edits = old_edits
                .into_iter()
                .map(|Indel { delete, insert }| {
                    TextEdit::new(document.line_index.line_col_lsp_range(delete), insert)
                })
                .collect();

            (uri.as_ref().clone(), new_edits)
        })
        .collect();

    Some(WorkspaceEdit::new(changes))
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Indel {
    delete: TextRange,
    insert: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct RenameResult {
    changes: FxHashMap<Arc<Url>, Vec<Indel>>,
}
