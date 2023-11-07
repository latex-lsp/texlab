use base_db::FeatureParams;
use rowan::{TextRange, TextSize};

mod label;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Highlight {
    pub range: TextRange,
    pub kind: HighlightKind,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum HighlightKind {
    Write,
    Read,
}

#[derive(Debug)]
pub struct HighlightParams<'a> {
    pub feature: FeatureParams<'a>,
    pub offset: TextSize,
}

pub fn find_all(params: HighlightParams) -> Vec<Highlight> {
    let mut results = Vec::new();
    label::find_highlights(&params, &mut results);
    results
}

#[cfg(test)]
mod tests;
