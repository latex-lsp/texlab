use std::collections::hash_map::HashMap;

use lsp_types::{RenameParams, TextEdit, WorkspaceEdit, ApplyWorkspaceEditParams};
use anyhow::Result;
use thiserror::Error;

use crate::{
    normalize_uri,
    util::{cursor::CursorContext, line_index_ext::LineIndexExt},
    Db
};

pub fn change_environment_context(db: &dyn Db, args: Vec<serde_json::Value>) -> Result<CursorContext<Params>> {
    let params: RenameParams =
        serde_json::from_value(
            args.into_iter().next().ok_or(ChangeEnvironmentError::MissingArg)?
            ).map_err(ChangeEnvironmentError::InvalidArg)?;

    let mut uri = params.text_document_position.text_document.uri;
    normalize_uri(&mut uri);
    let position = params.text_document_position.position;

    CursorContext::new(db, &uri, position, Params { new_name: params.new_name })
       .ok_or(ChangeEnvironmentError::FailedCreatiingContext.into())
}

pub fn change_environment(db: &dyn Db, context: &CursorContext<Params>) -> Option<ApplyWorkspaceEditParams> {
    let (beg_name, beg_range, end_name, end_range) = context.find_environment()?;
    if beg_name != end_name {
        return None;
    }
    let new_name = &context.params.new_name;

    let line_index = context.document.contents(db).line_index(db);
    let mut changes = HashMap::default();
    changes.insert(context.document.location(db).uri(db).clone(),
                   vec![
                       TextEdit::new(line_index.line_col_lsp_range(beg_range),
                           new_name.clone()),
                       TextEdit::new(line_index.line_col_lsp_range(end_range),
                           new_name.clone()),
                   ]);


    Some(ApplyWorkspaceEditParams{
            label: Some(format!("change environment: {} -> {}", beg_name, new_name)),
            edit: WorkspaceEdit::new(changes)
        })
}

#[derive(Debug, Error)]
pub enum ChangeEnvironmentError {
    #[error("rename parameters was not provided as an argument")]
    MissingArg,

    #[error("invalid argument: {0}")]
    InvalidArg(serde_json::Error),

    #[error("failed creating context")]
    FailedCreatiingContext,

    #[error("could not create workspaces edit")]
    CouldNotCreateWorkspaceEdit,
}

#[derive(Debug)]
pub struct Params {
    new_name: String,
}
