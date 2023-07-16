mod entry;
mod label;
mod string_def;

use base_db::{Document, Project, Workspace};
use rowan::{TextRange, TextSize};

#[derive(Debug)]
pub struct Reference<'db> {
    pub document: &'db Document,
    pub range: TextRange,
    pub kind: ReferenceKind,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum ReferenceKind {
    Definition,
    Reference,
}

#[derive(Debug)]
pub struct ReferenceParams<'db> {
    pub workspace: &'db Workspace,
    pub document: &'db Document,
    pub offset: TextSize,
}

#[derive(Debug)]
struct ReferenceContext<'db> {
    params: ReferenceParams<'db>,
    project: Project<'db>,
    items: Vec<Reference<'db>>,
}

pub fn find_all(params: ReferenceParams) -> Vec<Reference<'_>> {
    let project = params.workspace.project(params.document);
    let mut context = ReferenceContext {
        params,
        project,
        items: Vec::new(),
    };

    entry::find_all(&mut context);
    label::find_all(&mut context);
    string_def::find_all(&mut context);
    context.items
}

#[cfg(test)]
mod tests;
