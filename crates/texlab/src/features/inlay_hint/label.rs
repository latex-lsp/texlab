use rowan::TextRange;

use crate::{
    db::{analysis::label, Document},
    util::{self, label::LabeledObject},
    Db,
};

use super::InlayHintBuilder;

pub(super) fn find_hints(
    db: &dyn Db,
    document: Document,
    range: TextRange,
    builder: &mut InlayHintBuilder,
) -> Option<()> {
    let data = document.parse(db).as_tex()?;
    for label in data
        .analyze(db)
        .labels(db)
        .iter()
        .copied()
        .filter(|label| matches!(label.origin(db), label::Origin::Definition(_)))
        .filter(|label| label.range(db).intersect(range).is_some())
    {
        if let Some(rendered) = util::label::render(db, document, label) {
            if let Some(number) = &rendered.number {
                let text = match &rendered.object {
                    LabeledObject::Section { prefix, .. } => {
                        format!("{} {}", prefix, number.text(db))
                    }
                    LabeledObject::Float { kind, .. } => {
                        format!("{} {}", kind.as_str(), number.text(db))
                    }
                    LabeledObject::Theorem { kind, .. } => {
                        format!("{} {}", kind.text(db), number.text(db))
                    }
                    LabeledObject::Equation => format!("Equation ({})", number.text(db)),
                    LabeledObject::EnumItem => format!("Item {}", number.text(db)),
                };

                builder.push(label.range(db).end(), text);
            }
        }
    }

    Some(())
}
