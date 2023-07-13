use base_db::util::{queries, render_label};

use crate::{Hover, HoverData, HoverParams};

pub(super) fn find_hover<'db>(params: &'db HoverParams<'db>) -> Option<Hover<'db>> {
    let data = params.document.data.as_tex()?;
    let cursor = queries::object_at_cursor(&data.semantics.labels, params.offset)?;
    let definition = queries::definition(&params.project, &cursor.object.name.text)?;
    let label = render_label(&params.workspace, &params.project, definition)?;
    Some(Hover {
        range: cursor.range,
        data: HoverData::Label(label),
    })
}
