use base_db::{data::BibtexEntryTypeCategory, semantics::Span, Document};
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
        match self.kind {
            SymbolKind::Section => vec![&self.name, "latex", "section"],
            SymbolKind::Figure => vec![&self.name, "latex", "float", "figure"],
            SymbolKind::Algorithm => vec![&self.name, "latex", "float", "algorithm"],
            SymbolKind::Table => vec![&self.name, "latex", "float", "table"],
            SymbolKind::Listing => vec![&self.name, "latex", "float", "listing"],
            SymbolKind::Enumeration => vec![&self.name, "latex", "enumeration"],
            SymbolKind::EnumerationItem => vec![&self.name, "latex", "enumeration", "item"],
            SymbolKind::Theorem => vec![&self.name, "latex", "math"],
            SymbolKind::Equation => vec![&self.name, "latex", "math", "equation"],
            SymbolKind::Entry(BibtexEntryTypeCategory::String) => {
                vec![&self.name, "bibtex", "string"]
            }
            SymbolKind::Entry(_) => vec![&self.name, "bibtex", "entry"],
            SymbolKind::Field => vec![&self.name, "bibtex", "field"],
            SymbolKind::Environment => vec![&self.name, "latex", "environment"],
        }
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
