use rowan::ast::AstNode;
use syntax::bibtex::{self, HasName};
use text_size::TextRange;

use super::Span;

#[derive(Debug, Clone, Default)]
pub struct Semantics {
    pub entries: Vec<Entry>,
    pub strings: Vec<StringDef>,
}

impl Semantics {
    pub fn process_root(&mut self, root: &bibtex::SyntaxNode) {
        for node in root.children() {
            if let Some(entry) = bibtex::Entry::cast(node.clone()) {
                self.process_entry(entry);
            } else if let Some(string) = bibtex::StringDef::cast(node) {
                self.process_string_def(string);
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

    fn process_string_def(&mut self, string: bibtex::StringDef) {
        if let Some(name) = string.name_token() {
            self.strings.push(StringDef {
                name: Span {
                    range: name.text_range(),
                    text: name.text().into(),
                },
                full_range: string.syntax().text_range(),
            });
        }
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub name: Span,
    pub full_range: TextRange,
}

#[derive(Debug, Clone)]
pub struct StringDef {
    pub name: Span,
    pub full_range: TextRange,
}
