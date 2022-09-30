use lsp_types::{HoverParams, MarkupKind};

use crate::{component_db::COMPONENT_DATABASE, features::cursor::CursorContext, syntax::latex};

use super::HoverResult;

pub(super) fn find_component_hover(context: &CursorContext<HoverParams>) -> Option<HoverResult> {
    let document = context.request.main_document();
    let data = document.data().as_latex()?;
    for link in &data.extras.explicit_links {
        if matches!(
            link.kind,
            latex::ExplicitLinkKind::Package | latex::ExplicitLinkKind::Class
        ) && link.stem_range.contains_inclusive(context.offset)
        {
            let value = COMPONENT_DATABASE.documentation(&link.stem)?.value;
            return Some(HoverResult {
                value,
                value_kind: MarkupKind::PlainText,
                range: link.stem_range,
            });
        }
    }

    None
}
