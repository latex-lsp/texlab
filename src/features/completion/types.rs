use std::sync::Arc;

use rowan::TextRange;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

use crate::{features::lsp_kinds::Structure, BibtexEntryTypeDoc, BibtexFieldDoc, Uri};

#[derive(Debug, Clone)]
pub struct InternalCompletionItem<'a> {
    pub range: TextRange,
    pub data: InternalCompletionItemData<'a>,
    pub preselect: bool,
    pub score: Option<i64>,
}

impl<'a> InternalCompletionItem<'a> {
    pub fn new(range: TextRange, data: InternalCompletionItemData<'a>) -> Self {
        Self {
            range,
            data,
            preselect: false,
            score: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum InternalCompletionItemData<'a> {
    EntryType {
        ty: &'a BibtexEntryTypeDoc,
    },
    Field {
        field: &'a BibtexFieldDoc,
    },
    Argument {
        name: &'a str,
        image: Option<&'a str>,
    },
    BeginCommand,
    Citation {
        uri: Arc<Uri>,
        key: String,
        text: String,
        ty: Structure,
    },
    ComponentCommand {
        name: &'a SmolStr,
        image: Option<&'a str>,
        glyph: Option<&'a str>,
        file_names: &'a [SmolStr],
    },
    ComponentEnvironment {
        name: &'a SmolStr,
        file_names: &'a [SmolStr],
    },
    Class {
        name: SmolStr,
    },
    Package {
        name: SmolStr,
    },
    Color {
        name: &'a str,
    },
    ColorModel {
        name: &'a str,
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
        name: String,
        kind: Structure,
        header: Option<String>,
        footer: Option<String>,
        text: String,
    },
    UserCommand {
        name: &'a str,
    },
    UserEnvironment {
        name: &'a str,
    },
    PgfLibrary {
        name: &'a str,
    },
    TikzLibrary {
        name: &'a str,
    },
}

impl<'a> InternalCompletionItemData<'a> {
    pub fn label<'b: 'a>(&'b self) -> &'a str {
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
    Citation { uri: Uri, key: SmolStr },
    Argument,
    Acronym,
    GlossaryEntry,
}
