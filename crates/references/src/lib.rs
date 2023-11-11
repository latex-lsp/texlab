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
}

#[derive(Debug)]
struct ReferenceContext<'a> {
    params: ReferenceParams<'a>,
    results: Vec<Reference<'a>>,
}

pub fn find_all(params: ReferenceParams) -> Vec<Reference<'_>> {
    let mut context = ReferenceContext {
        params,
        results: Vec::new(),
    };

    entry::find_all(&mut context);
    label::find_all(&mut context);
    string_def::find_all(&mut context);
    context.results
}

#[cfg(test)]
mod tests;
