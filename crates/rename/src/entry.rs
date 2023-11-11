use base_db::{
    semantics::{bib, tex, Span},
    util::queries::{self},
    DocumentData,
};

use crate::{RenameBuilder, RenameParams};

pub(super) fn prepare_rename(params: &RenameParams) -> Option<Span> {
    match &params.feature.document.data {
        DocumentData::Tex(data) => {
            let result = queries::object_at_cursor(
                &data.semantics.citations,
                params.offset,
                queries::SearchMode::Name,
            )?;

            Some(Span::new(result.object.name.text.clone(), result.range))
        }
        DocumentData::Bib(data) => {
            let result = queries::object_at_cursor(
                &data.semantics.entries,
                params.offset,
                queries::SearchMode::Name,
            )?;

            Some(Span::new(result.object.name.text.clone(), result.range))
        }
        _ => None,
    }
}

pub(super) fn rename<'a>(builder: &mut RenameBuilder) -> Option<()> {
    let name = prepare_rename(&builder.params)?;

    let project = &builder.params.feature.project;
    let citations = queries::objects_with_name::<tex::Citation>(project, &name.text)
        .map(|(doc, obj)| (doc, obj.name.range));

    let entries = queries::objects_with_name::<bib::Entry>(project, &name.text)
        .map(|(doc, obj)| (doc, obj.name.range));

    for (document, range) in citations.chain(entries) {
        let entry = builder.result.changes.entry(document);
        entry.or_default().push(range);
    }

    Some(())
}
