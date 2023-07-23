use base_db::{
    semantics::tex,
    util::queries::{self, Object},
};

use crate::{Reference, ReferenceContext, ReferenceKind};

pub(super) fn find_all<'db>(context: &mut ReferenceContext<'db>) -> Option<()> {
    let data = context.params.document.data.as_tex()?;
    let mode = queries::SearchMode::Full;
    let name = queries::object_at_cursor(&data.semantics.labels, context.params.offset, mode)?
        .object
        .name_text();

    for (document, label) in queries::objects_with_name::<tex::Label>(&context.project, name) {
        let kind = match label.kind {
            tex::LabelKind::Definition => ReferenceKind::Definition,
            tex::LabelKind::Reference | tex::LabelKind::ReferenceRange => ReferenceKind::Reference,
        };

        context.results.push(Reference {
            document,
            range: label.name.range,
            kind,
        });
    }

    Some(())
}
