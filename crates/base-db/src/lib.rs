mod config;
pub mod data;
mod document;
pub mod graph;
pub mod semantics;
pub mod util;
mod workspace;

pub use self::{config::*, document::*, workspace::*};

#[derive(Debug)]
pub struct FeatureParams<'a> {
    pub document: &'a Document,
    pub project: Project<'a>,
    pub workspace: &'a Workspace,
}

impl<'a> FeatureParams<'a> {
    pub fn new(workspace: &'a Workspace, document: &'a Document) -> Self {
        let project = workspace.project(document);
        Self {
            document,
            project,
            workspace,
        }
    }
}
