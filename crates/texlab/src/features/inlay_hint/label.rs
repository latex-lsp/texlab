use base_db::{
    semantics::tex::{Label, LabelKind},
    util::{queries, render_label, RenderedLabel, RenderedObject},
    DocumentData,
};

use super::InlayHintBuilder;

pub(super) fn find_hints(builder: &mut InlayHintBuilder) -> Option<()> {
    let DocumentData::Tex(data) = &builder.document.data else {
        return None;
    };

    let range = builder.range;
    for label in data
        .semantics
        .labels
        .iter()
        .filter(|label| label.name.range.intersect(range).is_some())
    {
        let Some(rendered) = find_and_render(builder, &label.name.text) else {
            continue;
        };

        let Some(number) = &rendered.number else {
            continue;
        };

        let text = match &rendered.object {
            RenderedObject::Section { prefix, .. } => {
                format!("{} {}", prefix, number)
            }
            RenderedObject::Float { kind, .. } => {
                format!("{} {}", kind.as_str(), number)
            }
            RenderedObject::Theorem { kind, .. } => {
                format!("{} {}", kind, number)
            }
            RenderedObject::Equation => format!("Equation ({})", number),
            RenderedObject::EnumItem => format!("Item {}", number),
        };

        builder.push(label.full_range.end(), text);
    }

    Some(())
}

fn find_and_render<'a>(builder: &InlayHintBuilder<'a>, name: &str) -> Option<RenderedLabel<'a>> {
    let project = &builder.project;
    queries::objects_with_name::<Label>(project, name)
        .map(|(_, label)| label)
        .find(|label| label.kind == LabelKind::Definition)
        .and_then(|label| render_label(builder.workspace, project, label))
}
