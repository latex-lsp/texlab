use base_db::{semantics::tex::LabelKind, DocumentData};

use crate::util::{self, label::LabeledObject};

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
        let Some(rendered) = util::label::render(builder.workspace, &builder.related, label) else { continue };
        let Some(number) = &rendered.number else { continue };

        let text = match &rendered.object {
            LabeledObject::Section { prefix, .. } => {
                format!("{} {}", prefix, number)
            }
            LabeledObject::Float { kind, .. } => {
                format!("{} {}", kind.as_str(), number)
            }
            LabeledObject::Theorem { kind, .. } => {
                format!("{} {}", kind, number)
            }
            LabeledObject::Equation => format!("Equation ({})", number),
            LabeledObject::EnumItem => format!("Item {}", number),
        };

        builder.push(label.name.range.end(), text);
    }

    Some(())
}
