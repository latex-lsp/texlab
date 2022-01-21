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
            Self::Command => CompletionItemKind::FUNCTION,
            Self::Snippet => CompletionItemKind::SNIPPET,
            Self::Environment => CompletionItemKind::ENUM,
            Self::Section => CompletionItemKind::MODULE,
            Self::Float => CompletionItemKind::METHOD,
            Self::Theorem => CompletionItemKind::VARIABLE,
            Self::Equation => CompletionItemKind::CONSTANT,
            Self::Item => CompletionItemKind::ENUM_MEMBER,
            Self::Label => CompletionItemKind::CONSTRUCTOR,
            Self::Folder => CompletionItemKind::FOLDER,
            Self::File => CompletionItemKind::FILE,
            Self::PgfLibrary => CompletionItemKind::PROPERTY,
            Self::TikzLibrary => CompletionItemKind::PROPERTY,
            Self::Color => CompletionItemKind::COLOR,
            Self::ColorModel => CompletionItemKind::COLOR,
            Self::Package => CompletionItemKind::CLASS,
            Self::Class => CompletionItemKind::CLASS,
            Self::Entry(BibtexEntryTypeCategory::Misc) => CompletionItemKind::INTERFACE,
            Self::Entry(BibtexEntryTypeCategory::String) => CompletionItemKind::TEXT,
            Self::Entry(BibtexEntryTypeCategory::Article) => CompletionItemKind::EVENT,
            Self::Entry(BibtexEntryTypeCategory::Book) => CompletionItemKind::STRUCT,
            Self::Entry(BibtexEntryTypeCategory::Collection) => CompletionItemKind::TYPE_PARAMETER,
            Self::Entry(BibtexEntryTypeCategory::Part) => CompletionItemKind::OPERATOR,
            Self::Entry(BibtexEntryTypeCategory::Thesis) => CompletionItemKind::UNIT,
            Self::Field => CompletionItemKind::FIELD,
            Self::Argument => CompletionItemKind::VALUE,
            Self::GlossaryEntry => CompletionItemKind::KEYWORD,
        }
    }

    pub fn symbol_kind(self) -> SymbolKind {
        match self {
            Self::Command => SymbolKind::FUNCTION,
            Self::Snippet => unimplemented!(),
            Self::Environment => SymbolKind::ENUM,
            Self::Section => SymbolKind::MODULE,
            Self::Float => SymbolKind::METHOD,
            Self::Theorem => SymbolKind::VARIABLE,
            Self::Equation => SymbolKind::CONSTANT,
            Self::Item => SymbolKind::ENUM_MEMBER,
            Self::Label => SymbolKind::CONSTRUCTOR,
            Self::Folder => SymbolKind::NAMESPACE,
            Self::File => SymbolKind::FILE,
            Self::PgfLibrary => SymbolKind::PROPERTY,
            Self::TikzLibrary => SymbolKind::PROPERTY,
            Self::Color => unimplemented!(),
            Self::ColorModel => unimplemented!(),
            Self::Package => SymbolKind::CLASS,
            Self::Class => SymbolKind::CLASS,
            Self::Entry(BibtexEntryTypeCategory::Misc) => SymbolKind::INTERFACE,
            Self::Entry(BibtexEntryTypeCategory::String) => SymbolKind::STRING,
            Self::Entry(BibtexEntryTypeCategory::Article) => SymbolKind::EVENT,
            Self::Entry(BibtexEntryTypeCategory::Book) => SymbolKind::STRUCT,
            Self::Entry(BibtexEntryTypeCategory::Collection) => SymbolKind::TYPE_PARAMETER,
            Self::Entry(BibtexEntryTypeCategory::Part) => SymbolKind::OPERATOR,
            Self::Entry(BibtexEntryTypeCategory::Thesis) => SymbolKind::OBJECT,
            Self::Field => SymbolKind::FIELD,
            Self::Argument => SymbolKind::NUMBER,
            Self::GlossaryEntry => unimplemented!(),
        }
    }
}
