use lsp_types::Url;
use rowan::TextRange;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

use crate::{util::lsp_enums::Structure, BibtexEntryTypeDoc, BibtexFieldDoc};

#[derive(Debug, Clone)]
pub struct InternalCompletionItem<'db> {
    pub range: TextRange,
    pub data: InternalCompletionItemData<'db>,
    pub preselect: bool,
    pub score: Option<i64>,
}

impl<'db> InternalCompletionItem<'db> {
    pub fn new(range: TextRange, data: InternalCompletionItemData<'db>) -> Self {
        Self {
            range,
            data,
            preselect: false,
            score: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum InternalCompletionItemData<'db> {
    EntryType {
        ty: &'db BibtexEntryTypeDoc,
    },
    Field {
        field: &'db BibtexFieldDoc,
    },
    Argument {
        name: &'db str,
        image: Option<&'db str>,
    },
    BeginCommand,
    Citation {
        uri: Url,
        key: String,
        text: String,
        ty: Structure,
    },
    ComponentCommand {
        name: &'db SmolStr,
        image: Option<&'db str>,
        glyph: Option<&'db str>,
        file_names: &'db [SmolStr],
    },
    ComponentEnvironment {
        name: &'db SmolStr,
        file_names: &'db [SmolStr],
    },
    Class {
        name: SmolStr,
    },
    Package {
        name: SmolStr,
    },
    Color {
        name: &'db str,
    },
    ColorModel {
        name: &'db str,
    },
    Acronym {
        name: String,
    },
    GlossaryEntry {
        name: String,
    },
    File {
        name: SmolStr,
    },
    Directory {
        name: SmolStr,
    },
    Label {
        name: &'db str,
        kind: Structure,
        header: Option<String>,
        footer: Option<String>,
        text: String,
    },
    UserCommand {
        name: &'db str,
    },
    UserEnvironment {
        name: String,
    },
    PgfLibrary {
        name: &'db str,
    },
    TikzLibrary {
        name: &'db str,
    },
}

impl<'db> InternalCompletionItemData<'db> {
    pub fn label<'this: 'db>(&'this self) -> &'db str {
        match self {
            Self::EntryType { ty } => &ty.name,
            Self::Field { field } => &field.name,
            Self::Argument { name, .. } => name,
            Self::BeginCommand => "begin",
            Self::Citation { key, .. } => key,
            Self::ComponentCommand { name, .. } => name,
            Self::ComponentEnvironment { name, .. } => name,
            Self::Class { name } => name,
            Self::Package { name } => name,
            Self::Color { name } => name,
            Self::ColorModel { name } => name,
            Self::Acronym { name } => name,
            Self::GlossaryEntry { name } => name,
            Self::File { name } => name,
            Self::Directory { name } => name,
            Self::Label { name, .. } => name,
            Self::UserCommand { name } => name,
            Self::UserEnvironment { name } => name,
            Self::PgfLibrary { name } => name,
            Self::TikzLibrary { name } => name,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CompletionItemData {
    Command,
    CommandSnippet,
    Environment,
    Label,
    Folder,
    File,
    PgfLibrary,
    TikzLibrary,
    Color,
    ColorModel,
    Package,
    Class,
    EntryType,
    FieldName,
    Citation { uri: Url, key: SmolStr },
    Argument,
    Acronym,
    GlossaryEntry,
}
