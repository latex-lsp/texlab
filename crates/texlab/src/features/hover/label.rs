use base_db::semantics::tex::LabelKind;
use lsp_types::MarkupKind;

use crate::util::{self, cursor::CursorContext};

use super::HoverResult;

pub(super) fn find_hover(context: &CursorContext) -> Option<HoverResult> {
    let (name_text, range) = context
        .find_label_name_key()
        .or_else(|| context.find_label_name_command())?;

    context
        .related
        .iter()
        .filter_map(|document| document.data.as_tex())
        .flat_map(|data| data.semantics.labels.iter())
        .find(|label| label.kind == LabelKind::Definition && label.name.text == name_text)
        .and_then(|label| util::label::render(context.workspace, &context.related, label))
        .map(|label| HoverResult {
            range,
            value: label.reference(),
            value_kind: MarkupKind::PlainText,
        })
}
