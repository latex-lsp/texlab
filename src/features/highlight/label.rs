use lsp_types::{DocumentHighlight, DocumentHighlightKind};

use crate::{db::analysis::label, util::cursor::CursorContext, LineIndexExt};

pub fn find_label_highlights(context: &CursorContext) -> Option<Vec<DocumentHighlight>> {
    let db = context.db;
    let (name_text, _) = context.find_label_name_key()?;
    let data = context.document.parse(db).as_tex()?;

    let mut highlights = Vec::new();
    let line_index = context.document.contents(db).line_index(db);
    for label in data
        .analyze(db)
        .labels(db)
        .iter()
        .filter(|label| label.name(db).text(db) == &name_text)
    {
        let range = line_index.line_col_lsp_range(label.range(db));
        let kind = Some(match label.origin(db) {
            label::Origin::Definition(_) => DocumentHighlightKind::WRITE,
            label::Origin::Reference(_) => DocumentHighlightKind::READ,
            label::Origin::ReferenceRange(_) => DocumentHighlightKind::READ,
        });

        highlights.push(DocumentHighlight { range, kind });
    }

    Some(highlights)
}
