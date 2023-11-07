use base_db::semantics::tex::LabelKind;

use crate::{Highlight, HighlightKind, HighlightParams};

pub fn find_highlights(params: &HighlightParams, results: &mut Vec<Highlight>) -> Option<()> {
    let data = params.feature.document.data.as_tex()?;
    let labels = &data.semantics.labels;
    let cursor = labels
        .iter()
        .find(|label| label.name.range.contains(params.offset))?;

    for label in labels
        .iter()
        .filter(|label| label.name.text == cursor.name.text)
    {
        let range = label.name.range;
        let kind = match label.kind {
            LabelKind::Definition => HighlightKind::Write,
            LabelKind::Reference | LabelKind::ReferenceRange => HighlightKind::Read,
        };

        results.push(Highlight { range, kind });
    }

    Some(())
}
