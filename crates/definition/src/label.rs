use base_db::{
    semantics::tex,
    util::{
        queries::{self, Object},
        render_label,
    },
};

use crate::DefinitionContext;

use super::DefinitionResult;

pub(super) fn goto_definition(context: &mut DefinitionContext) -> Option<()> {
    let data = context.params.document.data.as_tex()?;
    let reference = queries::object_at_cursor(
        &data.semantics.labels,
        context.params.offset,
        queries::SearchMode::Full,
    )?;

    let name = reference.object.name_text();
    let labels = queries::objects_with_name::<tex::Label>(&context.project, name);
    for (document, label) in labels.filter(|(_, label)| label.kind == tex::LabelKind::Definition) {
        let target_selection_range = label.name.range;
        let target_range = render_label(context.params.workspace, &context.project, label)
            .map_or(target_selection_range, |label| label.range);

        context.results.insert(DefinitionResult {
            origin_selection_range: reference.object.name_range(),
            target: document,
            target_range,
            target_selection_range,
        });
    }

    Some(())
}
