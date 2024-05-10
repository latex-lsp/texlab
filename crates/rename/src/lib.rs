mod command;
mod entry;
mod label;

use base_db::{Document, FeatureParams};
use rowan::{TextRange, TextSize};
use rustc_hash::FxHashMap;

#[derive(Debug)]
pub struct RenameParams<'a> {
    pub feature: FeatureParams<'a>,
    pub offset: TextSize,
}

#[derive(Debug, Default)]
pub struct RenameResult<'a> {
    pub changes: FxHashMap<&'a Document, Vec<TextRange>>,
}

struct RenameBuilder<'a> {
    params: RenameParams<'a>,
    result: RenameResult<'a>,
}

pub fn prepare_rename(params: &RenameParams) -> Option<TextRange> {
    command::prepare_rename(params)
        .or_else(|| entry::prepare_rename(params))
        .or_else(|| label::prepare_rename(params))
        .map(|span| span.range)
}

pub fn rename<'a>(params: RenameParams<'a>) -> RenameResult<'a> {
    let result = RenameResult::default();
    let mut builder = RenameBuilder { params, result };

    command::rename(&mut builder)
        .or_else(|| entry::rename(&mut builder))
        .or_else(|| label::rename(&mut builder));

    builder.result
}

#[cfg(test)]
mod tests;
