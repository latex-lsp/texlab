mod label;

use lsp_types::{InlayHint, InlayHintLabel, Range, Url};
use rowan::TextSize;

use crate::{db::workspace::Workspace, Db, LineIndex, LineIndexExt};

pub fn find_all(db: &dyn Db, uri: &Url, range: Range) -> Option<Vec<InlayHint>> {
    let document = Workspace::get(db).lookup_uri(db, uri)?;
    let line_index = document.contents(db).line_index(db);

    let mut builder = InlayHintBuilder {
        line_index,
        hints: Vec::new(),
    };

    let range = line_index.offset_lsp_range(range);
    label::find_label_inlay_hints(db, document, range, &mut builder);
    Some(builder.hints)
}

struct InlayHintBuilder<'db> {
    line_index: &'db LineIndex,
    hints: Vec<InlayHint>,
}

impl<'db> InlayHintBuilder<'db> {
    pub fn push(&mut self, offset: TextSize, text: String) {
        let position = self.line_index.line_col_lsp(offset);
        self.hints.push(InlayHint {
            position,
            label: InlayHintLabel::String(text),
            kind: None,
            text_edits: None,
            tooltip: None,
            padding_left: Some(true),
            padding_right: None,
            data: None,
        });
    }
}
