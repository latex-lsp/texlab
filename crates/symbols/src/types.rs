use base_db::{Document, data::BibtexEntryTypeCategory, semantics::Span};
use rowan::TextRange;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SymbolKind {
    Section,
    Figure,
    Algorithm,
    Table,
    Listing,
    Enumeration,
    EnumerationItem,
    Theorem,
    Equation,
    Entry(BibtexEntryTypeCategory),
    Field,
    Environment,
    CommandDefinition,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub label: Option<Span>,
    pub full_range: TextRange,
    pub selection_range: TextRange,
    pub children: Vec<Symbol>,
}

impl Symbol {
    pub fn new_simple(
        name: String,
        kind: SymbolKind,
        full_range: TextRange,
        selection_range: TextRange,
    ) -> Self {
        Self {
            name,
            kind,
            label: None,
            full_range,
            selection_range,
            children: Vec::new(),
        }
    }

    pub fn new_label(name: String, kind: SymbolKind, range: TextRange, label: Span) -> Self {
        Self {
            name,
            kind,
            full_range: range,
            selection_range: label.range,
            label: Some(label),
            children: Vec::new(),
        }
    }

    pub fn keywords(&self) -> Vec<&str> {
        let name = self.name.split_whitespace();
        let tags = match self.kind {
            SymbolKind::Section => vec!["latex", "section"],
            SymbolKind::Figure => vec!["latex", "float", "figure"],
            SymbolKind::Algorithm => vec!["latex", "float", "algorithm"],
            SymbolKind::Table => vec!["latex", "float", "table"],
            SymbolKind::Listing => vec!["latex", "float", "listing"],
            SymbolKind::Enumeration => vec!["latex", "enumeration"],
            SymbolKind::EnumerationItem => vec!["latex", "enumeration", "item"],
            SymbolKind::Theorem => vec!["latex", "math"],
            SymbolKind::Equation => vec!["latex", "math", "equation"],
            SymbolKind::Entry(BibtexEntryTypeCategory::String) => vec!["bibtex", "string"],
            SymbolKind::Entry(_) => vec!["bibtex", "entry"],
            SymbolKind::Field => vec!["bibtex", "field"],
            SymbolKind::Environment => vec!["latex", "environment"],
            SymbolKind::CommandDefinition => vec!["latex", "command", "definition", "define"],
        };

        name.chain(tags).collect()
    }

    pub fn flatten(mut self, buffer: &mut Vec<Self>) {
        for symbol in self.children.drain(..) {
            symbol.flatten(buffer);
        }

        buffer.push(self);
    }
}

#[derive(Debug)]
pub struct SymbolLocation<'a> {
    pub document: &'a Document,
    pub symbol: Symbol,
}
