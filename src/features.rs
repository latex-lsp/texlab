pub mod building;
pub mod completion;
mod cursor;
pub mod definition;
mod execute_command;
pub mod folding;
pub mod formatting;
mod forward_search;
pub mod highlight;
pub mod hover;
pub mod inlay_hint;
pub mod link;
mod lsp_kinds;
pub mod reference;
pub mod rename;
pub mod symbol;

use std::sync::Arc;

use lsp_types::Url;

use crate::{Document, Workspace};

pub use self::{
    execute_command::execute_command,
    forward_search::{ForwardSearch, ForwardSearchResult, ForwardSearchStatus},
};

#[derive(Clone)]
pub struct FeatureRequest<P> {
    pub params: P,
    pub workspace: Workspace,
    pub uri: Arc<Url>,
}

impl<P> FeatureRequest<P> {
    pub fn main_document(&self) -> Document {
        self.workspace.get(&self.uri).unwrap()
    }
}
