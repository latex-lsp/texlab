use base_db::{
    DocumentData, DocumentLocation,
    semantics::{bib, tex},
    util::queries::{self, Object},
};

use crate::{Reference, ReferenceContext, ReferenceKind};

pub(super) fn find_all(context: &mut ReferenceContext) -> Option<()> {
    let offset = context.params.offset;

    let name = match &context.params.feature.document.data {
        DocumentData::Tex(data) => {
            let result = queries::object_at_cursor(
                &data.semantics.citations,
                offset,
                queries::SearchMode::Full,
            )?;
            result.object.name_text()
        }
        DocumentData::Bib(data) => {
            let result = queries::object_at_cursor(
                &data.semantics.entries,
                offset,
                queries::SearchMode::Name,
            )?;
            result.object.name_text()
        }
        _ => return None,
    };

    let project = &context.params.feature.project;
    for (document, obj) in queries::objects_with_name::<tex::Citation>(project, name) {
        context.results.push(Reference {
            location: DocumentLocation::new(document, obj.name.range),
            kind: ReferenceKind::Reference,
        });
    }

    for (document, obj) in queries::objects_with_name::<bib::Entry>(project, name) {
        context.results.push(Reference {
            location: DocumentLocation::new(document, obj.name.range),
            kind: ReferenceKind::Definition,
        });
    }

    Some(())
}
