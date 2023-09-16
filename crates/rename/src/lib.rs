mod command;
mod entry;
mod label;

use base_db::{Document, FeatureParams};
use rowan::{TextRange, TextSize};
use rustc_hash::FxHashMap;

#[derive(Debug)]
pub struct RenameParams<'db> {
    pub inner: FeatureParams<'db>,
    pub offset: TextSize,
}

#[derive(Debug, Default)]
pub struct RenameResult<'db> {
    pub changes: FxHashMap<&'db Document, Vec<TextRange>>,
}

struct RenameBuilder<'db> {
    params: &'db RenameParams<'db>,
    result: RenameResult<'db>,
}

pub fn prepare_rename(params: &RenameParams) -> Option<TextRange> {
    command::prepare_rename(&params)
        .or_else(|| entry::prepare_rename(&params))
        .or_else(|| label::prepare_rename(&params))
        .map(|span| span.range)
}

pub fn rename<'db>(params: &'db RenameParams<'db>) -> RenameResult<'db> {
    let result = RenameResult::default();
    let mut builder = RenameBuilder { params, result };

    command::rename(&mut builder)
        .or_else(|| entry::rename(&mut builder))
        .or_else(|| label::rename(&mut builder));

    builder.result
}

#[cfg(test)]
mod tests;
