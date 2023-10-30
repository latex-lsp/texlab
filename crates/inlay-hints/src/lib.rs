mod citations;
mod label;

use base_db::{util::RenderedLabel, FeatureParams};
use rowan::{TextRange, TextSize};

pub struct InlayHintParams<'a> {
    pub range: TextRange,
    pub feature: FeatureParams<'a>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InlayHint<'a> {
    pub offset: TextSize,
    pub data: InlayHintData<'a>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InlayHintData<'a> {
    LabelDefinition(RenderedLabel<'a>),
    LabelReference(RenderedLabel<'a>),
    Citation(String),
}

pub fn find_all<'a>(params: InlayHintParams<'a>) -> Option<Vec<InlayHint>> {
    let mut builder = InlayHintBuilder {
        params,
        hints: Vec::new(),
    };

    label::find_hints(&mut builder);
    citations::find_hints(&mut builder);
    Some(builder.hints)
}

struct InlayHintBuilder<'a> {
    params: InlayHintParams<'a>,
    hints: Vec<InlayHint<'a>>,
}
