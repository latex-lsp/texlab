mod citation;
mod entry_type;
mod field_type;
mod label;
mod package;
mod string_ref;

use base_db::{
    data::{BibtexEntryType, BibtexFieldType},
    util::RenderedLabel,
    FeatureParams,
};
use rowan::{TextRange, TextSize};

#[derive(Debug)]
pub struct HoverParams<'a> {
    pub feature: FeatureParams<'a>,
    pub offset: TextSize,
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

pub fn find<'a>(params: &HoverParams<'a>) -> Option<Hover<'a>> {
    citation::find_hover(params)
        .or_else(|| package::find_hover(params))
        .or_else(|| entry_type::find_hover(params))
        .or_else(|| field_type::find_hover(params))
        .or_else(|| label::find_hover(params))
        .or_else(|| string_ref::find_hover(params))
}

#[cfg(test)]
mod tests;
