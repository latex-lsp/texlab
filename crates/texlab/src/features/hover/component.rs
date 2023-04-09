use base_db::{semantics::tex::LinkKind, DocumentData};
use lsp_types::MarkupKind;

use crate::util::{components::COMPONENT_DATABASE, cursor::CursorContext};

use super::HoverResult;

pub(super) fn find_hover(context: &CursorContext) -> Option<HoverResult> {
    let DocumentData::Tex(data) = &context.document.data else { return None };
    data.semantics
        .links
        .iter()
        .filter(|link| matches!(link.kind, LinkKind::Sty | LinkKind::Cls))
        .filter(|link| link.path.range.contains_inclusive(context.offset))
        .find_map(|link| {
            let value = COMPONENT_DATABASE.documentation(&link.path.text)?.value;
            Some(HoverResult {
                value,
                value_kind: MarkupKind::PlainText,
                range: link.path.range,
            })
        })
}
