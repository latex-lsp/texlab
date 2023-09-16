use base_db::{
    semantics::{tex, Span},
    util::queries::{self, Object},
};

use crate::{RenameBuilder, RenameParams};

pub(super) fn prepare_rename(params: &RenameParams) -> Option<Span> {
    let data = params.inner.document.data.as_tex()?;
    let labels = &data.semantics.labels;
    let label = queries::object_at_cursor(labels, params.offset, queries::SearchMode::Name)?;
    Some(Span::new(label.object.name.text.clone(), label.range))
}

pub(super) fn rename(builder: &mut RenameBuilder) -> Option<()> {
    let name = prepare_rename(&builder.params)?;

    let project = &builder.params.inner.project;
    for (document, label) in queries::objects_with_name::<tex::Label>(project, &name.text) {
        let entry = builder.result.changes.entry(document);
        entry.or_default().push(label.name_range());
    }

    Some(())
}
