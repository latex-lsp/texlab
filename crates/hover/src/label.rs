use base_db::{
    semantics::tex,
    util::{
        queries::{self, Object},
        render_label,
    },
};

use crate::{Hover, HoverData, HoverParams};

pub(super) fn find_hover<'a>(params: &HoverParams<'a>) -> Option<Hover<'a>> {
    let feature = &params.feature;
    let data = feature.document.data.as_tex()?;
    let cursor = queries::object_at_cursor(
        &data.semantics.labels,
        params.offset,
        queries::SearchMode::Full,
    )?;

    let (_, definition) = tex::Label::find_all(&feature.project)
        .filter(|(_, label)| label.kind == tex::LabelKind::Definition)
        .find(|(_, label)| label.name_text() == cursor.object.name_text())?;

    let label = render_label(feature.workspace, &feature.project, definition)?;
    Some(Hover {
        range: cursor.range,
        data: HoverData::Label(label),
    })
}
