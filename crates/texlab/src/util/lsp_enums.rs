use base_db::data::BibtexEntryTypeCategory;
use lsp_types::CompletionItemKind;

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
}
