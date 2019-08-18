use crate::syntax::*;
use lsp_types::{MarkupContent, MarkupKind};

pub fn render_citation(entry_code: &str) -> Option<MarkupContent> {
    let tree = BibtexSyntaxTree::from(entry_code);
    if tree.entries().iter().any(|entry| entry.fields.len() == 0) {
        return None;
    }

    let markdown = citeproc::render(entry_code)?;
    Some(MarkupContent {
        kind: MarkupKind::Markdown,
        value: markdown.trim().to_owned().into(),
    })
}
