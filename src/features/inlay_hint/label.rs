use rowan::TextRange;

use crate::{
    db::{analysis::label, Document},
    util, Db,
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
            builder.push(label.range(db).end(), rendered.reference(db));
        }
    }

    Some(())
}
