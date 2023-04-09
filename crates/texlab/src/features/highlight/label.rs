use base_db::{semantics::tex::LabelKind, DocumentData};
use lsp_types::{DocumentHighlight, DocumentHighlightKind};

use crate::util::{cursor::CursorContext, line_index_ext::LineIndexExt};

pub fn find_highlights(context: &CursorContext) -> Option<Vec<DocumentHighlight>> {
    let (name_text, _) = context.find_label_name_key()?;

    let DocumentData::Tex(data) = &context.document.data else { return None };

    let mut highlights = Vec::new();
    let line_index = &context.document.line_index;
    for label in data
        .semantics
        .labels
        .iter()
        .filter(|label| label.name.text == name_text)
    {
        let range = line_index.line_col_lsp_range(label.name.range);
        let kind = Some(match label.kind {
            LabelKind::Definition => DocumentHighlightKind::WRITE,
            LabelKind::Reference => DocumentHighlightKind::READ,
            LabelKind::ReferenceRange => DocumentHighlightKind::READ,
        });

        highlights.push(DocumentHighlight { range, kind });
    }

    Some(highlights)
}
