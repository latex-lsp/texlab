use base_db::{semantics::tex::LabelKind, DocumentData};
use lsp_types::ReferenceContext;

use crate::util::cursor::CursorContext;

use super::ReferenceResult;

pub(super) fn find_all_references<'a>(
    context: &CursorContext<'a, &ReferenceContext>,
    results: &mut Vec<ReferenceResult<'a>>,
) -> Option<()> {
    let (name_text, _) = context
        .find_label_name_key()
        .or_else(|| context.find_label_name_command())?;

    for document in &context.related {
        let DocumentData::Tex(data) = &document.data else { continue };

        for label in data
            .semantics
            .labels
            .iter()
            .filter(|label| label.name.text == name_text)
            .filter(|label| {
                label.kind != LabelKind::Definition || context.params.include_declaration
            })
        {
            results.push(ReferenceResult {
                document,
                range: label.name.range,
            });
        }
    }

    Some(())
}
