use base_db::{semantics::tex::LabelKind, Document};
use lsp_types::DocumentHighlight;
use rowan::TextSize;

use crate::util::line_index_ext::LineIndexExt;

pub fn find_highlights(
    document: &Document,
    offset: TextSize,
) -> Option<Vec<lsp_types::DocumentHighlight>> {
    let data = document.data.as_tex()?;
    let cursor = data
        .semantics
        .labels
        .iter()
        .find(|label| label.name.range.contains(offset))?;

    let mut highlights = Vec::new();
    let line_index = &document.line_index;
    for label in data
        .semantics
        .labels
        .iter()
        .filter(|label| label.name.text == cursor.name.text)
    {
        let range = line_index.line_col_lsp_range(label.name.range);
        let kind = Some(match label.kind {
            LabelKind::Definition => lsp_types::DocumentHighlightKind::WRITE,
            LabelKind::Reference => lsp_types::DocumentHighlightKind::READ,
            LabelKind::ReferenceRange => lsp_types::DocumentHighlightKind::READ,
        });

        highlights.push(DocumentHighlight { range, kind });
    }

    Some(highlights)
}
