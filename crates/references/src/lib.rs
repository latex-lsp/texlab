mod command;
mod entry;
mod label;
mod string_def;

use base_db::{DocumentLocation, FeatureParams};
use rowan::TextSize;

#[derive(Debug)]
pub struct Reference<'a> {
    pub location: DocumentLocation<'a>,
    pub kind: ReferenceKind,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum ReferenceKind {
    Definition,
    Reference,
}

#[derive(Debug)]
pub struct ReferenceParams<'a> {
    pub feature: FeatureParams<'a>,
    pub offset: TextSize,
    pub include_declaration: bool,
}

#[derive(Debug)]
struct ReferenceContext<'a, 'b> {
    params: &'b ReferenceParams<'a>,
    results: Vec<Reference<'a>>,
}

pub fn find_all<'a>(params: &ReferenceParams<'a>) -> Vec<DocumentLocation<'a>> {
    let mut context = ReferenceContext {
        params,
        results: Vec::new(),
    };

    entry::find_all(&mut context);
    label::find_all(&mut context);
    string_def::find_all(&mut context);
    command::find_all(&mut context);

    context
        .results
        .into_iter()
        .filter(|r| r.kind == ReferenceKind::Reference || params.include_declaration)
        .map(|reference| reference.location)
        .collect()
}

#[cfg(test)]
mod tests;
