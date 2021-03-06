mod api;
mod children_expand;
mod document;
mod parent_expand;
mod storage;
mod watch;

use std::sync::Arc;

use anyhow::Result;

use crate::ServerContext;

pub use self::{api::*, document::*};
use self::{
    children_expand::ChildrenExpander, parent_expand::ParentExpander, storage::Storage,
    watch::DocumentWatcher,
};

pub fn create_workspace_fast(context: Arc<ServerContext>) -> Result<impl Workspace> {
    let workspace = Storage::new(context);
    let workspace = ChildrenExpander::new(Arc::new(workspace));
    Ok(workspace)
}

pub fn create_workspace_full(context: Arc<ServerContext>) -> Result<impl Workspace> {
    let workspace = Storage::new(context);
    let workspace = DocumentWatcher::new(Arc::new(workspace))?;
    let workspace = ParentExpander::new(workspace);
    let workspace = ChildrenExpander::new(Arc::new(workspace));
    Ok(workspace)
}
