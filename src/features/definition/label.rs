use crate::util::{self, cursor::CursorContext};

use super::DefinitionResult;

pub(super) fn goto_definition(context: &CursorContext) -> Option<Vec<DefinitionResult>> {
    let db = context.db;
    let (name_text, origin_selection_range) = context
        .find_label_name_key()
        .or_else(|| context.find_label_name_command())?;

    for document in context.related() {
        if let Some(data) = document.parse(db).as_tex() {
            if let Some(label) = data
                .analyze(db)
                .labels(db)
                .iter()
                .find(|label| label.name(db).text(db) == name_text.as_str())
            {
                let target_selection_range = label.range(db);
                let target_range = util::label::render(db, document, *label)
                    .map_or(target_selection_range, |label| label.range);

                return Some(vec![DefinitionResult {
                    origin_selection_range,
                    target: document,
                    target_range,
                    target_selection_range,
                }]);
            }
        }
    }

    None
}
