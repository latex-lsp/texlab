mod citation;
mod command;
mod include;
mod label;
mod string_ref;

use base_db::{Document, FeatureParams};
use rowan::{TextRange, TextSize};
use rustc_hash::FxHashSet;

#[derive(Debug)]
pub struct DefinitionParams<'a> {
    pub feature: FeatureParams<'a>,
    pub offset: TextSize,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct DefinitionResult<'a> {
    pub origin_selection_range: TextRange,
    pub target: &'a Document,
    pub target_range: TextRange,
    pub target_selection_range: TextRange,
}

#[derive(Debug)]
struct DefinitionContext<'a> {
    params: DefinitionParams<'a>,
    results: FxHashSet<DefinitionResult<'a>>,
}

pub fn goto_definition(params: DefinitionParams) -> FxHashSet<DefinitionResult> {
    let mut context = DefinitionContext {
        params,
        results: FxHashSet::default(),
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
