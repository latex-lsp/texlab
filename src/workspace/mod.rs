mod api;
mod children_expand;
mod document;
mod parent_expand;
mod storage;

use std::sync::Arc;

use anyhow::Result;

use crate::ServerContext;

pub use self::{api::*, document::*};
use self::{children_expand::ChildrenExpander, parent_expand::ParentExpander, storage::Storage};

pub fn create_workspace(context: Arc<ServerContext>) -> Result<impl Workspace> {
    let workspace = Storage::new(context);
    let workspace = ParentExpander::new(workspace);
    let workspace = ChildrenExpander::new(Arc::new(workspace));
    Ok(workspace)
}
