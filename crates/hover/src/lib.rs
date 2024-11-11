mod citation;
mod command;
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
pub struct Hover<'a> {
    pub range: TextRange,
    pub data: HoverData<'a>,
}

#[derive(Debug, Clone)]
pub enum HoverData<'a> {
    Citation(String),
    Package(&'a str),
    Command(&'a completion_data::Command<'static>),
    EntryType(BibtexEntryType<'a>),
    FieldType(BibtexFieldType<'a>),
    Label(RenderedLabel<'a>),
    StringRef(String),
}

pub fn find<'a>(params: &HoverParams<'a>) -> Option<Hover<'a>> {
    citation::find_hover(params)
        .or_else(|| package::find_hover(params))
        .or_else(|| entry_type::find_hover(params))
        .or_else(|| field_type::find_hover(params))
        .or_else(|| label::find_hover(params))
        .or_else(|| string_ref::find_hover(params))
        .or_else(|| command::find_hover(params))
}

#[cfg(test)]
mod tests;
