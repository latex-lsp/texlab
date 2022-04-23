use lsp_types::{DocumentSymbol, Location, Range, SymbolInformation, SymbolKind, Url};

use crate::{features::lsp_kinds::Structure, BibtexEntryTypeCategory};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InternalSymbolKind {
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
    String,
}

impl InternalSymbolKind {
    pub fn into_symbol_kind(self) -> SymbolKind {
        match self {
            Self::Section => Structure::Section.symbol_kind(),
            Self::Figure | Self::Algorithm | Self::Table | Self::Listing => {
                Structure::Float.symbol_kind()
            }
            Self::Enumeration => Structure::Environment.symbol_kind(),
            Self::EnumerationItem => Structure::Item.symbol_kind(),
            Self::Theorem => Structure::Theorem.symbol_kind(),
            Self::Equation => Structure::Equation.symbol_kind(),
            Self::Entry(category) => Structure::Entry(category).symbol_kind(),
            Self::Field => Structure::Field.symbol_kind(),
            Self::String => Structure::Entry(BibtexEntryTypeCategory::String).symbol_kind(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InternalSymbol {
    pub name: String,
    pub label: Option<String>,
    pub kind: InternalSymbolKind,
    pub deprecated: bool,
    pub full_range: Range,
    pub selection_range: Range,
    pub children: Vec<InternalSymbol>,
}

impl InternalSymbol {
    pub fn search_text(&self) -> String {
        let kind = match self.kind {
            InternalSymbolKind::Section => "latex section",
            InternalSymbolKind::Figure => "latex float figure",
            InternalSymbolKind::Algorithm => "latex float algorithm",
            InternalSymbolKind::Table => "latex float table",
            InternalSymbolKind::Listing => "latex float listing",
            InternalSymbolKind::Enumeration => "latex enumeration",
            InternalSymbolKind::EnumerationItem => "latex enumeration item",
            InternalSymbolKind::Theorem => "latex math",
            InternalSymbolKind::Equation => "latex math equation",
            InternalSymbolKind::Entry(_) => "bibtex entry",
            InternalSymbolKind::Field => "bibtex field",
            InternalSymbolKind::String => "bibtex string",
        };
        format!("{} {}", kind, self.name).to_lowercase()
    }

    pub fn flatten(mut self, buffer: &mut Vec<Self>) {
        if self.kind == InternalSymbolKind::Field {
            return;
        }
        for symbol in self.children.drain(..) {
            symbol.flatten(buffer);
        }
        buffer.push(self);
    }

    pub fn into_document_symbol(self) -> DocumentSymbol {
        let children = self
            .children
            .into_iter()
            .map(|child| child.into_document_symbol())
            .collect();
        #[allow(deprecated)]
        DocumentSymbol {
            name: self.name,
            detail: self.label,
            kind: self.kind.into_symbol_kind(),
            deprecated: Some(self.deprecated),
            range: self.full_range,
            selection_range: self.selection_range,
            children: Some(children),
            tags: None,
        }
    }

    pub fn into_symbol_info(self, uri: Url) -> SymbolInformation {
        #[allow(deprecated)]
        SymbolInformation {
            name: self.name,
            kind: self.kind.into_symbol_kind(),
            deprecated: Some(self.deprecated),
            location: Location::new(uri, self.full_range),
            container_name: None,
            tags: None,
        }
    }
}
