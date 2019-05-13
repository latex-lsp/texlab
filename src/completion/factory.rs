use crate::formatting::{BibtexFormatter, BibtexFormattingOptions};
use crate::metadata::bibtex_entry_type::BibtexEntryType;
use crate::metadata::bibtex_field::BibtexField;
use crate::syntax::bibtex::{BibtexEntry, BibtexToken};
use lsp_types::*;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::path::Path;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LatexComponentId {
    Kernel,
    Unknown,
    User(Vec<String>),
}

impl LatexComponentId {
    fn detail(&self) -> Cow<'static, str> {
        match self {
            LatexComponentId::Kernel => Cow::from("built-in"),
            LatexComponentId::Unknown => Cow::from("unknown"),
            LatexComponentId::User(files) => Cow::from(files.join(", ")),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum CompletionItemData {
    Snippet,
    Command,
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
    Citation,
    CommandSymbol,
    ArgumentSymbol,
}

impl Into<serde_json::Value> for CompletionItemData {
    fn into(self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}

pub fn create_snippet(
    name: Cow<'static, str>,
    component: &LatexComponentId,
    template: Cow<'static, str>,
) -> CompletionItem {
    CompletionItem {
        kind: Some(CompletionItemKind::Snippet),
        data: Some(CompletionItemData::Snippet.into()),
        insert_text: Some(template),
        insert_text_format: Some(InsertTextFormat::Snippet),
        ..CompletionItem::new_simple(name, component.detail())
    }
}

pub fn create_command(name: Cow<'static, str>, component: &LatexComponentId) -> CompletionItem {
    CompletionItem {
        kind: Some(CompletionItemKind::Function),
        data: Some(CompletionItemData::Command.into()),
        ..CompletionItem::new_simple(name, component.detail())
    }
}

pub fn create_environment(name: Cow<'static, str>, component: &LatexComponentId) -> CompletionItem {
    CompletionItem {
        kind: Some(CompletionItemKind::EnumMember),
        data: Some(CompletionItemData::Environment.into()),
        ..CompletionItem::new_simple(name, component.detail())
    }
}

pub fn create_label(name: Cow<'static, str>) -> CompletionItem {
    CompletionItem {
        label: name,
        kind: Some(CompletionItemKind::Field),
        data: Some(CompletionItemData::Label.into()),
        ..CompletionItem::default()
    }
}

pub fn create_folder(path: &Path) -> CompletionItem {
    CompletionItem {
        label: Cow::from(path.file_name().unwrap().to_string_lossy().into_owned()),
        kind: Some(CompletionItemKind::Folder),
        data: Some(CompletionItemData::Folder.into()),
        ..CompletionItem::default()
    }
}

pub fn create_file(path: &Path) -> CompletionItem {
    CompletionItem {
        label: Cow::from(path.file_name().unwrap().to_string_lossy().into_owned()),
        kind: Some(CompletionItemKind::File),
        data: Some(CompletionItemData::File.into()),
        ..CompletionItem::default()
    }
}

pub fn create_pgf_library(name: Cow<'static, str>) -> CompletionItem {
    CompletionItem {
        label: name,
        kind: Some(CompletionItemKind::Module),
        data: Some(CompletionItemData::PgfLibrary.into()),
        ..CompletionItem::default()
    }
}

pub fn create_tikz_library(name: Cow<'static, str>) -> CompletionItem {
    CompletionItem {
        label: name,
        kind: Some(CompletionItemKind::Color),
        data: Some(CompletionItemData::Color.into()),
        ..CompletionItem::default()
    }
}

pub fn create_color(name: Cow<'static, str>) -> CompletionItem {
    CompletionItem {
        label: name,
        kind: Some(CompletionItemKind::Color),
        data: Some(CompletionItemData::Color.into()),
        ..CompletionItem::default()
    }
}

pub fn create_color_model(name: Cow<'static, str>) -> CompletionItem {
    CompletionItem {
        label: name,
        kind: Some(CompletionItemKind::Color),
        data: Some(CompletionItemData::ColorModel.into()),
        ..CompletionItem::default()
    }
}

pub fn create_package(name: Cow<'static, str>) -> CompletionItem {
    CompletionItem {
        label: name,
        kind: Some(CompletionItemKind::Class),
        data: Some(CompletionItemData::Package.into()),
        ..CompletionItem::default()
    }
}

pub fn create_class(name: Cow<'static, str>) -> CompletionItem {
    CompletionItem {
        label: name,
        kind: Some(CompletionItemKind::Class),
        data: Some(CompletionItemData::Class.into()),
        ..CompletionItem::default()
    }
}

pub fn create_citation(entry: &BibtexEntry, key: &str) -> CompletionItem {
    let mut formatter = BibtexFormatter::new(BibtexFormattingOptions::new(2, true, 35));
    formatter.format_entry(entry);
    let markdown = format!("```bibtex\n{}\n```", formatter.output);
    CompletionItem {
        label: Cow::from(key.to_owned()),
        kind: Some(CompletionItemKind::Field),
        data: Some(CompletionItemData::Citation.into()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: Cow::from(markdown),
        })),
        ..CompletionItem::default()
    }
}

pub fn create_entry_type(ty: &BibtexEntryType) -> CompletionItem {
    CompletionItem {
        label: Cow::from(ty.name),
        kind: Some(CompletionItemKind::Interface),
        data: Some(CompletionItemData::EntryType.into()),
        documentation: ty.documentation.map(|documentation| {
            Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: Cow::from(documentation),
            })
        }),
        ..CompletionItem::default()
    }
}

pub fn create_field_name(field: &BibtexField) -> CompletionItem {
    CompletionItem {
        label: Cow::from(field.name),
        kind: Some(CompletionItemKind::Field),
        data: Some(CompletionItemData::FieldName.into()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: Cow::from(field.documentation),
        })),
        ..CompletionItem::default()
    }
}
