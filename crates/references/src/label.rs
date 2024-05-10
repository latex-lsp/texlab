use base_db::{
    semantics::tex,
    util::queries::{self, Object},
    DocumentLocation,
};

use crate::{Reference, ReferenceContext, ReferenceKind};

pub(super) fn find_all(context: &mut ReferenceContext) -> Option<()> {
    let data = context.params.feature.document.data.as_tex()?;
    let mode = queries::SearchMode::Full;
    let name = queries::object_at_cursor(&data.semantics.labels, context.params.offset, mode)?
        .object
        .name_text();

    let project = &context.params.feature.project;
    for (document, label) in queries::objects_with_name::<tex::Label>(project, name) {
        let kind = match label.kind {
            tex::LabelKind::Definition => ReferenceKind::Definition,
            tex::LabelKind::Reference | tex::LabelKind::ReferenceRange => ReferenceKind::Reference,
        };

        context.results.push(Reference {
            location: DocumentLocation::new(document, label.name.range),
            kind,
        });
    }

    Some(())
}
