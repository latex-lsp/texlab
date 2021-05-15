use lsp_types::{CompletionItemKind, SymbolKind};

use crate::BibtexEntryTypeCategory;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Structure {
    Command,
    Snippet,
    Environment,
    Section,
    Float,
    Theorem,
    Equation,
    Item,
    Label,
    Folder,
    File,
    PgfLibrary,
    TikzLibrary,
    Color,
    ColorModel,
    Package,
    Class,
    Entry(BibtexEntryTypeCategory),
    Field,
    Argument,
    GlossaryEntry,
}

impl Structure {
    pub fn completion_kind(self) -> CompletionItemKind {
        match self {
            Self::Command => CompletionItemKind::Function,
            Self::Snippet => CompletionItemKind::Snippet,
            Self::Environment => CompletionItemKind::Enum,
            Self::Section => CompletionItemKind::Module,
            Self::Float => CompletionItemKind::Method,
            Self::Theorem => CompletionItemKind::Variable,
            Self::Equation => CompletionItemKind::Constant,
            Self::Item => CompletionItemKind::EnumMember,
            Self::Label => CompletionItemKind::Constructor,
            Self::Folder => CompletionItemKind::Folder,
            Self::File => CompletionItemKind::File,
            Self::PgfLibrary => CompletionItemKind::Property,
            Self::TikzLibrary => CompletionItemKind::Property,
            Self::Color => CompletionItemKind::Color,
            Self::ColorModel => CompletionItemKind::Color,
            Self::Package => CompletionItemKind::Class,
            Self::Class => CompletionItemKind::Class,
            Self::Entry(BibtexEntryTypeCategory::Misc) => CompletionItemKind::Interface,
            Self::Entry(BibtexEntryTypeCategory::String) => CompletionItemKind::Text,
            Self::Entry(BibtexEntryTypeCategory::Article) => CompletionItemKind::Event,
            Self::Entry(BibtexEntryTypeCategory::Book) => CompletionItemKind::Struct,
            Self::Entry(BibtexEntryTypeCategory::Collection) => CompletionItemKind::TypeParameter,
            Self::Entry(BibtexEntryTypeCategory::Part) => CompletionItemKind::Operator,
            Self::Entry(BibtexEntryTypeCategory::Thesis) => CompletionItemKind::Unit,
            Self::Field => CompletionItemKind::Field,
            Self::Argument => CompletionItemKind::Value,
            Self::GlossaryEntry => CompletionItemKind::Keyword,
        }
    }

    pub fn symbol_kind(self) -> SymbolKind {
        match self {
            Self::Command => SymbolKind::Function,
            Self::Snippet => unimplemented!(),
            Self::Environment => SymbolKind::Enum,
            Self::Section => SymbolKind::Module,
            Self::Float => SymbolKind::Method,
            Self::Theorem => SymbolKind::Variable,
            Self::Equation => SymbolKind::Constant,
            Self::Item => SymbolKind::EnumMember,
            Self::Label => SymbolKind::Constructor,
            Self::Folder => SymbolKind::Namespace,
            Self::File => SymbolKind::File,
            Self::PgfLibrary => SymbolKind::Property,
            Self::TikzLibrary => SymbolKind::Property,
            Self::Color => unimplemented!(),
            Self::ColorModel => unimplemented!(),
            Self::Package => SymbolKind::Class,
            Self::Class => SymbolKind::Class,
            Self::Entry(BibtexEntryTypeCategory::Misc) => SymbolKind::Interface,
            Self::Entry(BibtexEntryTypeCategory::String) => SymbolKind::String,
            Self::Entry(BibtexEntryTypeCategory::Article) => SymbolKind::Event,
            Self::Entry(BibtexEntryTypeCategory::Book) => SymbolKind::Struct,
            Self::Entry(BibtexEntryTypeCategory::Collection) => SymbolKind::TypeParameter,
            Self::Entry(BibtexEntryTypeCategory::Part) => SymbolKind::Operator,
            Self::Entry(BibtexEntryTypeCategory::Thesis) => SymbolKind::Object,
            Self::Field => SymbolKind::Field,
            Self::Argument => SymbolKind::Number,
            Self::GlossaryEntry => unimplemented!(),
        }
    }
}
