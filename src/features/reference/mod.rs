mod entry;
mod label;
mod string;

use lsp_types::{Location, ReferenceParams};

use self::{
    entry::find_entry_references, label::find_label_references, string::find_string_references,
};

use super::{cursor::CursorContext, FeatureRequest};

pub fn find_all_references(request: FeatureRequest<ReferenceParams>) -> Option<Vec<Location>> {
    let mut references = Vec::new();
    let context = CursorContext::new(request);
    log::debug!("[References] Cursor: {:?}", context.cursor);
    find_label_references(&context, &mut references);
    find_entry_references(&context, &mut references);
    find_string_references(&context, &mut references);
    Some(references)
}
