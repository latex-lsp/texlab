use lsp_types::{HoverParams, MarkupKind};

use crate::{features::cursor::CursorContext, render_label};

use super::HoverResult;

pub(super) fn find_label_hover(context: &CursorContext<HoverParams>) -> Option<HoverResult> {
    let (name_text, range) = context
        .find_label_name_key()
        .or_else(|| context.find_label_name_command())?;

    let label = render_label(&context.request.workspace, &name_text, None)?;

    Some(HoverResult {
        range,
        value: label.reference(),
        value_kind: MarkupKind::PlainText,
    })
}
