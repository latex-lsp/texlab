use lsp_types::ReferenceContext;

use crate::util::cursor::CursorContext;

use super::ReferenceResult;

pub(super) fn find_all_references(
    context: &CursorContext<&ReferenceContext>,
    results: &mut Vec<ReferenceResult>,
) -> Option<()> {
    let db = context.db;
    let (name_text, _) = context
        .find_label_name_key()
        .or_else(|| context.find_label_name_command())?;

    for document in context.related() {
        if let Some(data) = document.parse(db).as_tex() {
            for label in data
                .analyze(db)
                .labels(db)
                .iter()
                .filter(|label| label.name(db).text(db) == &name_text)
                .filter(|label| {
                    label.origin(db).as_definition().is_none() || context.params.include_declaration
                })
            {
                results.push(ReferenceResult {
                    document,
                    range: label.range(db),
                });
            }
        }
    }

    Some(())
}
