mod label;

use base_db::{Document, Workspace};
use lsp_types::{InlayHint, InlayHintLabel, Range, Url};
use rowan::{TextRange, TextSize};
use rustc_hash::FxHashSet;

use crate::util::line_index_ext::LineIndexExt;

pub fn find_all(workspace: &Workspace, uri: &Url, range: Range) -> Option<Vec<InlayHint>> {
    let document = workspace.lookup(uri)?;
    let range = document.line_index.offset_lsp_range(range);
    let related = workspace.related(document);

    let mut builder = InlayHintBuilder {
        workspace,
        document,
        related,
        range,
        hints: Vec::new(),
    };

    label::find_hints(&mut builder);
    Some(builder.hints)
}

struct InlayHintBuilder<'a> {
    workspace: &'a Workspace,
    document: &'a Document,
    related: FxHashSet<&'a Document>,
    range: TextRange,
    hints: Vec<InlayHint>,
}

impl<'db> InlayHintBuilder<'db> {
    pub fn push(&mut self, offset: TextSize, text: String) {
        let position = self.document.line_index.line_col_lsp(offset);
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
