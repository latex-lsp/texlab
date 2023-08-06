mod citation;
mod command;
mod include;
mod label;
mod string_ref;

use base_db::{Document, Project, Workspace};
use rowan::{TextRange, TextSize};

#[derive(Debug)]
pub struct DefinitionParams<'db> {
    pub workspace: &'db Workspace,
    pub document: &'db Document,
    pub offset: TextSize,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DefinitionResult<'a> {
    pub origin_selection_range: TextRange,
    pub target: &'a Document,
    pub target_range: TextRange,
    pub target_selection_range: TextRange,
}

#[derive(Debug)]
struct DefinitionContext<'db> {
    params: DefinitionParams<'db>,
    project: Project<'db>,
    results: Vec<DefinitionResult<'db>>,
}

pub fn goto_definition(params: DefinitionParams) -> Vec<DefinitionResult> {
    let project = params.workspace.project(params.document);
    let mut context = DefinitionContext {
        params,
        project,
        results: Vec::new(),
    };

    command::goto_definition(&mut context);
    include::goto_definition(&mut context);
    citation::goto_definition(&mut context);
    label::goto_definition(&mut context);
    string_ref::goto_definition(&mut context);
    context.results
}

#[cfg(test)]
mod tests;
