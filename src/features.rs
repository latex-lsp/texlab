pub mod building;
mod completion;
mod cursor;
mod definition;
mod execute_command;
pub mod folding;
pub mod formatting;
mod forward_search;
mod highlight;
pub mod hover;
pub mod inlay_hint;
pub mod link;
mod lsp_kinds;
mod reference;
mod rename;
mod symbol;

use std::sync::Arc;

use lsp_types::Url;

use crate::{Document, Workspace};

pub use self::{
    completion::{complete, CompletionItemData, COMPLETION_LIMIT},
    definition::goto_definition,
    execute_command::execute_command,
    forward_search::{ForwardSearch, ForwardSearchResult, ForwardSearchStatus},
    highlight::find_document_highlights,
    reference::find_all_references,
    rename::{prepare_rename_all, rename_all},
    symbol::{find_document_symbols, find_workspace_symbols},
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
