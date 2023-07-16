use base_db::{
    semantics::tex::LabelKind,
    util::queries::{self, Object},
    DocumentData,
};

use crate::{Reference, ReferenceContext, ReferenceKind};

pub(super) fn find_all<'db>(context: &mut ReferenceContext<'db>) -> Option<()> {
    let data = context.params.document.data.as_tex()?;
    let mode = queries::SearchMode::Full;
    let name = queries::object_at_cursor(&data.semantics.labels, context.params.offset, mode)?
        .object
        .name_text();

    for document in &context.project.documents {
        let DocumentData::Tex(data) = &document.data else {
            continue;
        };

        let labels = data.semantics.labels.iter();
        for label in labels.filter(|label| label.name.text == name) {
            let kind = match label.kind {
                LabelKind::Definition => ReferenceKind::Definition,
                LabelKind::Reference | LabelKind::ReferenceRange => ReferenceKind::Reference,
            };

            context.items.push(Reference {
                document,
                range: label.name.range,
                kind,
            });
        }
    }

    Some(())
}
