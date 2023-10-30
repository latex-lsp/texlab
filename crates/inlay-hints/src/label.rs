use base_db::{semantics::tex::LabelKind, util::render_label};

use crate::InlayHintData;

use super::InlayHintBuilder;

pub(super) fn find_hints(builder: &mut InlayHintBuilder) -> Option<()> {
    let params = &builder.params.feature;
    let data = params.document.data.as_tex()?;
    let range = builder.params.range;
    for label in data
        .semantics
        .labels
        .iter()
        .filter(|label| label.kind == LabelKind::Definition)
        .filter(|label| label.name.range.intersect(range).is_some())
    {
        let Some(rendered) = render_label(params.workspace, &params.project, label) else {
            continue;
        };

        builder.hints.push(crate::InlayHint {
            offset: label.full_range.end(),
            data: InlayHintData::LabelDefinition(rendered),
        });
    }

    Some(())
}
