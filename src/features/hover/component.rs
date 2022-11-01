use lsp_types::MarkupKind;

use crate::{
    component_db::COMPONENT_DATABASE, db::analysis::TexLinkKind, util::cursor::CursorContext,
};

use super::HoverResult;

pub(super) fn find_component_hover(context: &CursorContext) -> Option<HoverResult> {
    let db = context.db;
    let links = context.document.parse(db).as_tex()?.analyze(db).links(db);
    links
        .iter()
        .filter(|link| matches!(link.kind(db), TexLinkKind::Sty | TexLinkKind::Cls))
        .filter(|link| link.range(db).contains_inclusive(context.offset))
        .find_map(|link| {
            let value = COMPONENT_DATABASE
                .documentation(link.path(db).text(db))?
                .value;

            Some(HoverResult {
                value,
                value_kind: MarkupKind::PlainText,
                range: link.range(db),
            })
        })
}
