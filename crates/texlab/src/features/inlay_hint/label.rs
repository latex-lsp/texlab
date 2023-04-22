use base_db::{
    semantics::tex::LabelKind,
    util::{render_label, RenderedObject},
    DocumentData,
};

use super::InlayHintBuilder;

pub(super) fn find_hints(builder: &mut InlayHintBuilder) -> Option<()> {
    let DocumentData::Tex(data) = &builder.document.data else { return None };

    let range = builder.range;
    for label in data
        .semantics
        .labels
        .iter()
        .filter(|label| label.kind == LabelKind::Definition)
        .filter(|label| label.name.range.intersect(range).is_some())
    {
        let Some(rendered) = render_label(builder.workspace, &builder.project, label) else { continue };
        let Some(number) = &rendered.number else { continue };

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

        builder.push(label.name.range.end(), text);
    }

    Some(())
}
