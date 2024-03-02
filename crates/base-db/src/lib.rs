mod config;
pub mod data;
pub mod deps;
mod document;
pub mod semantics;
pub mod util;
mod workspace;

pub use self::{config::*, document::*, workspace::*};

#[derive(Debug)]
pub struct FeatureParams<'a> {
    pub document: &'a Document,
    pub project: deps::Project<'a>,
    pub workspace: &'a Workspace,
}

impl<'a> FeatureParams<'a> {
    pub fn new(workspace: &'a Workspace, document: &'a Document) -> Self {
        let project = deps::Project::from_child(workspace, document);
        Self {
            document,
            project,
            workspace,
        }
    }
}
