mod citation;
mod entry_type;
mod field_type;
mod label;
mod package;
mod string_ref;

use base_db::{
    data::{BibtexEntryType, BibtexFieldType},
    util::RenderedLabel,
    Document, Project, Workspace,
};
use rowan::{TextRange, TextSize};

#[derive(Debug)]
pub struct HoverParams<'db> {
    pub document: &'db Document,
    pub project: Project<'db>,
    pub workspace: &'db Workspace,
    pub offset: TextSize,
}

impl<'db> HoverParams<'db> {
    pub fn new(workspace: &'db Workspace, document: &'db Document, offset: TextSize) -> Self {
        let project = workspace.project(document);
        Self {
            document,
            project,
            workspace,
            offset,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Hover<'db> {
    pub range: TextRange,
    pub data: HoverData<'db>,
}

#[derive(Debug, Clone)]
pub enum HoverData<'db> {
    Citation(String),
    Package(&'db str),
    EntryType(BibtexEntryType<'db>),
    FieldType(BibtexFieldType<'db>),
    Label(RenderedLabel<'db>),
    StringRef(String),
}

pub fn find<'db>(params: &'db HoverParams<'db>) -> Option<Hover<'db>> {
    citation::find_hover(params)
        .or_else(|| package::find_hover(params))
        .or_else(|| entry_type::find_hover(params))
        .or_else(|| field_type::find_hover(params))
        .or_else(|| label::find_hover(params))
        .or_else(|| string_ref::find_hover(params))
}

#[cfg(test)]
mod tests;
