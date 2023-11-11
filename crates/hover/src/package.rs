use base_db::semantics::tex::LinkKind;

use crate::{Hover, HoverData, HoverParams};

pub(super) fn find_hover<'a>(params: &HoverParams<'a>) -> Option<Hover<'a>> {
    let data = params.feature.document.data.as_tex()?;
    data.semantics
        .links
        .iter()
        .filter(|link| matches!(link.kind, LinkKind::Sty | LinkKind::Cls))
        .filter(|link| link.path.range.contains_inclusive(params.offset))
        .find_map(|link| {
            let meta = completion_data::DATABASE.meta(&link.path.text)?;
            let description = meta.description.as_deref()?;
            Some(Hover {
                range: link.path.range,
                data: HoverData::Package(description),
            })
        })
}
