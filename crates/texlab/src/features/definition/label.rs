use base_db::{semantics::tex::LabelKind, util::render_label, DocumentData};

use crate::util::cursor::CursorContext;

use super::DefinitionResult;

pub(super) fn goto_definition<'a>(
    context: &CursorContext<'a>,
) -> Option<Vec<DefinitionResult<'a>>> {
    let (name_text, origin_selection_range) = context
        .find_label_name_key()
        .or_else(|| context.find_label_name_command())?;

    for document in &context.project.documents {
        let DocumentData::Tex(data) = &document.data else { continue };

        let Some(label) = data
            .semantics
            .labels
            .iter()
            .filter(|label| label.kind == LabelKind::Definition)
            .find(|label| label.name.text == name_text) else { continue };

        let target_selection_range = label.name.range;
        let target_range = render_label(context.workspace, &context.project, label)
            .map_or(target_selection_range, |label| label.range);

        return Some(vec![DefinitionResult {
            origin_selection_range,
            target: document,
            target_range,
            target_selection_range,
        }]);
    }

    None
}
