mod build;
#[cfg(feature = "completion")]
mod completion;
mod cursor;
mod definition;
mod execute_command;
mod folding;
mod formatting;
mod forward_search;
mod highlight;
mod hover;
mod link;
mod lsp_kinds;
mod reference;
mod rename;
mod symbol;

use std::sync::Arc;

use lsp_types::Url;

use crate::{Document, Workspace};

#[cfg(feature = "completion")]
pub use self::completion::{complete, CompletionItemData, COMPLETION_LIMIT};
pub use self::{
    build::{BuildEngine, BuildParams, BuildResult, BuildStatus},
    definition::goto_definition,
    execute_command::execute_command,
    folding::find_foldings,
    formatting::format_source_code,
    forward_search::{execute_forward_search, ForwardSearchResult, ForwardSearchStatus},
    highlight::find_document_highlights,
    hover::find_hover,
    link::find_document_links,
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
    pub fn main_document(&self) -> &Document {
        &self.workspace.documents_by_uri[&self.uri]
    }
}
