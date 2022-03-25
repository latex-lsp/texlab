mod command;
mod entry;
mod label;

use cancellation::CancellationToken;
use lsp_types::{Range, RenameParams, TextDocumentPositionParams, WorkspaceEdit};

use self::{
    command::{prepare_command_rename, rename_command},
    entry::{prepare_entry_rename, rename_entry},
    label::{prepare_label_rename, rename_label},
};

use super::{cursor::CursorContext, FeatureRequest};

pub fn prepare_rename_all(
    request: FeatureRequest<TextDocumentPositionParams>,
    cancellation_token: &CancellationToken,
) -> Option<Range> {
    let context = CursorContext::new(request);
    prepare_entry_rename(&context, cancellation_token)
        .or_else(|| prepare_label_rename(&context, cancellation_token))
        .or_else(|| prepare_command_rename(&context, cancellation_token))
}

pub fn rename_all(
    request: FeatureRequest<RenameParams>,
    cancellation_token: &CancellationToken,
) -> Option<WorkspaceEdit> {
    let context = CursorContext::new(request);
    rename_entry(&context, cancellation_token)
        .or_else(|| rename_label(&context, cancellation_token))
        .or_else(|| rename_command(&context, cancellation_token))
}
