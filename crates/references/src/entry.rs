use base_db::{
    util::queries::{self, Object},
    DocumentData,
};

use crate::{Reference, ReferenceContext, ReferenceKind};

pub(super) fn find_all<'db>(context: &mut ReferenceContext<'db>) -> Option<()> {
    let offset = context.params.offset;

    let name = match &context.params.document.data {
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

    for document in &context.project.documents {
        if let DocumentData::Tex(data) = &document.data {
            let citations = data.semantics.citations.iter();
            for citation in citations.filter(|citation| citation.name.text == name) {
                context.items.push(Reference {
                    document,
                    range: citation.name.range,
                    kind: ReferenceKind::Reference,
                });
            }
        } else if let DocumentData::Bib(data) = &document.data {
            let entries = data.semantics.entries.iter();
            for entry in entries.filter(|entry| entry.name.text == name) {
                context.items.push(Reference {
                    document,
                    range: entry.name.range,
                    kind: ReferenceKind::Definition,
                });
            }
        }
    }

    Some(())
}
