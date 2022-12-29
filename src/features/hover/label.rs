use lsp_types::MarkupKind;

use crate::{
    db::Word,
    util::{self, cursor::CursorContext},
};

use super::HoverResult;

pub(super) fn find_hover(context: &CursorContext) -> Option<HoverResult> {
    let (name_text, range) = context
        .find_label_name_key()
        .or_else(|| context.find_label_name_command())?;

    let db = context.db;
    util::label::find_label_definition(db, context.document, Word::new(db, name_text))
        .and_then(|(document, label)| util::label::render(db, document, label))
        .map(|label| HoverResult {
            range,
            value: label.reference(db),
            value_kind: MarkupKind::PlainText,
        })
}
