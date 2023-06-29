use rowan::ast::AstNode;
use syntax::bibtex::{self, HasName};
use text_size::TextRange;

use super::Span;

#[derive(Debug, Clone, Default)]
pub struct Semantics {
    pub entries: Vec<Entry>,
}

impl Semantics {
    pub fn process_root(&mut self, root: &bibtex::SyntaxNode) {
        for node in root.children() {
            if let Some(entry) = bibtex::Entry::cast(node) {
                self.process_entry(entry);
            }
        }
    }

    fn process_entry(&mut self, entry: bibtex::Entry) {
        if let Some(name) = entry.name_token() {
            self.entries.push(Entry {
                name: Span {
                    range: name.text_range(),
                    text: name.text().into(),
                },
                full_range: entry.syntax().text_range(),
            });
        }
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub name: Span,
    pub full_range: TextRange,
}
