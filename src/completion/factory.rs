use crate::data::label::LabelContext;
use crate::data::language::{BibtexEntryType, BibtexField};
use crate::feature::FeatureRequest;
use crate::formatting::bibtex::{self, BibtexFormattingParams};
use crate::syntax::bibtex::BibtexEntry;
use lsp_types::*;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::path::Path;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
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
    Citation { entry_code: String },
    Argument,
}

impl Into<serde_json::Value> for CompletionItemData {
    fn into(self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LatexComponentId<'a> {
    User,
    Component(Vec<&'a str>),
}

impl<'a> LatexComponentId<'a> {
    pub fn kernel() -> Self {
        LatexComponentId::Component(vec![])
    }

    pub fn detail(&self) -> Cow<'static, str> {
        match self {
            LatexComponentId::User => "user-defined".into(),
            LatexComponentId::Component(files) => {
                if files.is_empty() {
                    "built-in".into()
                } else {
                    files.join(", ").into()
                }
            }
        }
    }
}

pub fn command(
    request: &FeatureRequest<CompletionParams>,
    name: Cow<'static, str>,
    image: Option<&str>,
    text_edit: TextEdit,
    component: &LatexComponentId,
) -> CompletionItem {
    CompletionItem {
        kind: Some(adjust_kind(request, CompletionItemKind::Function)),
        data: Some(CompletionItemData::Command.into()),
        documentation: image.map(|image| image_documentation(&name, image)),
        text_edit: Some(text_edit),
        ..CompletionItem::new_simple(name, component.detail())
    }
}

pub fn command_snippet(
    request: &FeatureRequest<CompletionParams>,
    name: &'static str,
    image: Option<&str>,
    template: &'static str,
    component: &LatexComponentId,
) -> CompletionItem {
    CompletionItem {
        kind: Some(adjust_kind(request, CompletionItemKind::Snippet)),
        data: Some(CompletionItemData::CommandSnippet.into()),
        documentation: image.map(|image| image_documentation(&name, image)),
        insert_text: Some(template.into()),
        insert_text_format: Some(InsertTextFormat::Snippet),
        ..CompletionItem::new_simple(name.into(), component.detail())
    }
}

pub fn environment(
    request: &FeatureRequest<CompletionParams>,
    name: Cow<'static, str>,
    text_edit: TextEdit,
    component: &LatexComponentId,
) -> CompletionItem {
    CompletionItem {
        kind: Some(adjust_kind(request, CompletionItemKind::EnumMember)),
        data: Some(CompletionItemData::Environment.into()),
        text_edit: Some(text_edit),
        ..CompletionItem::new_simple(name, component.detail())
    }
}

pub fn label(
    request: &FeatureRequest<CompletionParams>,
    name: Cow<'static, str>,
    text_edit: TextEdit,
    context: &LabelContext,
) -> CompletionItem {
    fn to_str(value: &Option<String>) -> &str {
        value.as_ref().map(String::as_str).unwrap_or_default()
    }

    let filter_text = format!(
        "{} {} {}",
        &name,
        to_str(&context.caption),
        to_str(&context.section)
    );

    CompletionItem {
        label: name,
        kind: Some(adjust_kind(request, CompletionItemKind::Field)),
        data: Some(CompletionItemData::Label.into()),
        text_edit: Some(text_edit),
        filter_text: Some(filter_text.into()),
        documentation: context.documentation().map(Documentation::MarkupContent),
        ..CompletionItem::default()
    }
}

pub fn folder(
    request: &FeatureRequest<CompletionParams>,
    path: &Path,
    text_edit: TextEdit,
) -> CompletionItem {
    CompletionItem {
        label: path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned()
            .into(),
        kind: Some(adjust_kind(request, CompletionItemKind::Folder)),
        data: Some(CompletionItemData::Folder.into()),
        text_edit: Some(text_edit),
        ..CompletionItem::default()
    }
}

pub fn file(
    request: &FeatureRequest<CompletionParams>,
    path: &Path,
    text_edit: TextEdit,
) -> CompletionItem {
    CompletionItem {
        label: path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned()
            .into(),
        kind: Some(adjust_kind(request, CompletionItemKind::File)),
        data: Some(CompletionItemData::File.into()),
        text_edit: Some(text_edit),
        ..CompletionItem::default()
    }
}

pub fn pgf_library(
    request: &FeatureRequest<CompletionParams>,
    name: &'static str,
    text_edit: TextEdit,
) -> CompletionItem {
    CompletionItem {
        label: name.into(),
        kind: Some(adjust_kind(request, CompletionItemKind::Module)),
        data: Some(CompletionItemData::PgfLibrary.into()),
        text_edit: Some(text_edit),
        ..CompletionItem::default()
    }
}

pub fn tikz_library(
    request: &FeatureRequest<CompletionParams>,
    name: &'static str,
    text_edit: TextEdit,
) -> CompletionItem {
    CompletionItem {
        label: name.into(),
        kind: Some(adjust_kind(request, CompletionItemKind::Module)),
        data: Some(CompletionItemData::TikzLibrary.into()),
        text_edit: Some(text_edit),
        ..CompletionItem::default()
    }
}

pub fn color(
    request: &FeatureRequest<CompletionParams>,
    name: &'static str,
    text_edit: TextEdit,
) -> CompletionItem {
    CompletionItem {
        label: name.into(),
        kind: Some(adjust_kind(request, CompletionItemKind::Color)),
        data: Some(CompletionItemData::Color.into()),
        text_edit: Some(text_edit),
        ..CompletionItem::default()
    }
}

pub fn color_model(
    request: &FeatureRequest<CompletionParams>,
    name: &'static str,
    text_edit: TextEdit,
) -> CompletionItem {
    CompletionItem {
        label: name.into(),
        kind: Some(adjust_kind(request, CompletionItemKind::Color)),
        data: Some(CompletionItemData::ColorModel.into()),
        text_edit: Some(text_edit),
        ..CompletionItem::default()
    }
}

pub fn package(
    request: &FeatureRequest<CompletionParams>,
    name: &'static str,
    text_edit: TextEdit,
) -> CompletionItem {
    CompletionItem {
        label: name.into(),
        kind: Some(adjust_kind(request, CompletionItemKind::Class)),
        data: Some(CompletionItemData::Package.into()),
        text_edit: Some(text_edit),
        ..CompletionItem::default()
    }
}

pub fn class(
    request: &FeatureRequest<CompletionParams>,
    name: &'static str,
    text_edit: TextEdit,
) -> CompletionItem {
    CompletionItem {
        label: name.into(),
        kind: Some(adjust_kind(request, CompletionItemKind::Class)),
        data: Some(CompletionItemData::Class.into()),
        text_edit: Some(text_edit),
        ..CompletionItem::default()
    }
}

pub fn citation(
    request: &FeatureRequest<CompletionParams>,
    entry: &BibtexEntry,
    key: String,
    text_edit: TextEdit,
) -> CompletionItem {
    let params = BibtexFormattingParams::default();
    let entry_code = bibtex::format_entry(&entry, &params);
    CompletionItem {
        label: key.into(),
        kind: Some(adjust_kind(request, CompletionItemKind::Field)),
        filter_text: Some(entry_code.replace('{', "").replace('}', "").into()),
        data: Some(CompletionItemData::Citation { entry_code }.into()),
        text_edit: Some(text_edit),
        ..CompletionItem::default()
    }
}

pub fn entry_type(
    request: &FeatureRequest<CompletionParams>,
    ty: &'static BibtexEntryType,
    text_edit: TextEdit,
) -> CompletionItem {
    CompletionItem {
        label: (&ty.name).into(),
        kind: Some(adjust_kind(request, CompletionItemKind::Interface)),
        data: Some(CompletionItemData::EntryType.into()),
        text_edit: Some(text_edit),
        documentation: ty.documentation.as_ref().map(|doc| {
            Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: doc.into(),
            })
        }),
        ..CompletionItem::default()
    }
}

pub fn field_name(
    request: &FeatureRequest<CompletionParams>,
    field: &'static BibtexField,
    text_edit: TextEdit,
) -> CompletionItem {
    CompletionItem {
        label: (&field.name).into(),
        kind: Some(adjust_kind(request, CompletionItemKind::Field)),
        data: Some(CompletionItemData::FieldName.into()),
        text_edit: Some(text_edit),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: (&field.documentation).into(),
        })),
        ..CompletionItem::default()
    }
}

pub fn argument(
    request: &FeatureRequest<CompletionParams>,
    name: &'static str,
    text_edit: TextEdit,
    image: Option<&str>,
) -> CompletionItem {
    CompletionItem {
        label: name.into(),
        kind: Some(adjust_kind(request, CompletionItemKind::Field)),
        data: Some(CompletionItemData::Argument.into()),
        text_edit: Some(text_edit),
        documentation: image.map(|image| image_documentation(&name, image)),
        ..CompletionItem::default()
    }
}

fn image_documentation(name: &str, image: &str) -> Documentation {
    Documentation::MarkupContent(MarkupContent {
        kind: MarkupKind::Markdown,
        value: format!(
            "![{}](data:image/png;base64,{}|width=48,height=48)",
            name, image
        )
        .into(),
    })
}

fn adjust_kind(
    request: &FeatureRequest<CompletionParams>,
    kind: CompletionItemKind,
) -> CompletionItemKind {
    if let Some(value_set) = request
        .client_capabilities
        .text_document
        .as_ref()
        .and_then(|cap| cap.completion.as_ref())
        .and_then(|cap| cap.completion_item_kind.as_ref())
        .and_then(|cap| cap.value_set.as_ref())
    {
        if value_set.contains(&kind) {
            return kind;
        }
    }
    CompletionItemKind::Text
}
