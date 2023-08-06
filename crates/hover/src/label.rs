use base_db::{
    semantics::tex,
    util::{
        queries::{self, Object},
        render_label,
    },
};

use crate::{Hover, HoverData, HoverParams};

pub(super) fn find_hover<'db>(params: &'db HoverParams<'db>) -> Option<Hover<'db>> {
    let data = params.document.data.as_tex()?;
    let cursor = queries::object_at_cursor(
        &data.semantics.labels,
        params.offset,
        queries::SearchMode::Full,
    )?;

    let (_, definition) = tex::Label::find_all(&params.project)
        .filter(|(_, label)| label.kind == tex::LabelKind::Definition)
        .find(|(_, label)| label.name_text() == cursor.object.name_text())?;

    let label = render_label(params.workspace, &params.project, definition)?;
    Some(Hover {
        range: cursor.range,
        data: HoverData::Label(label),
    })
}
