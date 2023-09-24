use base_db::{
    util::{LineIndex, RenderedObject},
    Document, FeatureParams, Workspace,
};
use completion::{
    ArgumentData, CompletionItem, CompletionItemData, CompletionParams, EntryTypeData,
    FieldTypeData,
};
use lsp_types::{ClientCapabilities, ClientInfo, CompletionList};
use serde::{Deserialize, Serialize};

use crate::util::{
    capabilities::ClientCapabilitiesExt, line_index_ext::LineIndexExt, lsp_enums::Structure,
};

pub fn complete(
    workspace: &Workspace,
    params: &lsp_types::CompletionParams,
    client_capabilities: &ClientCapabilities,
    client_info: Option<&ClientInfo>,
) -> Option<CompletionList> {
    let document = workspace.lookup(&params.text_document_position.text_document.uri)?;
    let feature = FeatureParams::new(workspace, document);
    let offset = document
        .line_index
        .offset_lsp(params.text_document_position.position);

    let params = CompletionParams { feature, offset };
    let result = completion::complete(&params);

    let mut list = CompletionList::default();
    let item_builder = ItemBuilder::new(document, client_capabilities);
    let always_incomplete = client_info.map_or(false, |info| info.name == "Visual Studio Code");
    list.is_incomplete = always_incomplete || result.items.len() >= completion::LIMIT;
    list.items = result
        .items
        .into_iter()
        .enumerate()
        .map(|(i, item)| item_builder.convert(item, i))
        .collect();

    Some(list)
}

struct ItemBuilder<'a> {
    line_index: &'a LineIndex,
    item_kinds: &'a [lsp_types::CompletionItemKind],
    supports_snippets: bool,
    supports_images: bool,
}

impl<'a> ItemBuilder<'a> {
    pub fn new(document: &'a Document, client_capabilities: &'a ClientCapabilities) -> Self {
        let line_index = &document.line_index;
        let item_kinds = client_capabilities
            .text_document
            .as_ref()
            .and_then(|cap| cap.completion.as_ref())
            .and_then(|cap| cap.completion_item_kind.as_ref())
            .and_then(|cap| cap.value_set.as_deref())
            .unwrap_or_default();

        let supports_snippets = client_capabilities.has_snippet_support();
        let supports_images = client_capabilities.has_completion_markdown_support();
        Self {
            line_index,
            item_kinds,
            supports_snippets,
            supports_images,
        }
    }

    pub fn convert(&self, item: CompletionItem, index: usize) -> lsp_types::CompletionItem {
        let mut result = lsp_types::CompletionItem::default();
        let range = self.line_index.line_col_lsp_range(item.range);

        match item.data {
            CompletionItemData::Command(data) => {
                self.convert_command(&mut result, range, data);
            }
            CompletionItemData::BeginEnvironment => {
                self.convert_begin_environment(&mut result, range);
            }
            CompletionItemData::Citation(data) => {
                self.convert_citation(&mut result, range, data);
            }
            CompletionItemData::Environment(data) => {
                self.convert_environment(&mut result, range, data);
            }
            CompletionItemData::GlossaryEntry(data) => {
                self.convert_glossary_entry(&mut result, range, data);
            }
            CompletionItemData::Label(data) => {
                self.convert_label(&mut result, range, data);
            }
            CompletionItemData::Color(name) => {
                self.convert_color(&mut result, range, name);
            }
            CompletionItemData::ColorModel(name) => {
                self.convert_color_model(&mut result, range, name);
            }
            CompletionItemData::File(name) => {
                self.convert_file(&mut result, range, name);
            }
            CompletionItemData::Directory(name) => {
                self.convert_directory(&mut result, range, name);
            }
            CompletionItemData::Argument(data) => {
                self.convert_argument(&mut result, range, data);
            }
            CompletionItemData::Package(name) => {
                self.convert_package(&mut result, range, name);
            }
            CompletionItemData::DocumentClass(name) => {
                self.convert_document_class(&mut result, range, name);
            }
            CompletionItemData::EntryType(data) => {
                self.convert_entry_type(&mut result, range, data);
            }
            CompletionItemData::Field(data) => {
                self.convert_field(&mut result, range, data);
            }
            CompletionItemData::TikzLibrary(name) => {
                self.convert_tikz_library(&mut result, range, name);
            }
        }

        if result
            .kind
            .is_some_and(|kind| !self.item_kinds.contains(&kind))
        {
            result.kind = Some(lsp_types::CompletionItemKind::TEXT);
        }

        result.sort_text = Some(format!("{:0>2}", index));
        result.preselect = Some(item.preselect);
        result
    }

    fn convert_command(
        &self,
        result: &mut lsp_types::CompletionItem,
        range: lsp_types::Range,
        data: completion::CommandData<'_>,
    ) {
        let origin = data.package.map_or_else(
            || String::from("user-defined"),
            |pkg| format_package_files(&pkg.file_names),
        );

        let detail = match &data.glyph {
            Some(glyph) => format!("{glyph}, {origin}"),
            None => origin,
        };

        result.label = data.name.into();
        result.detail = Some(detail);
        result.kind = Some(Structure::Command.completion_kind());
        result.documentation = data
            .image
            .and_then(|base64| self.inline_image(data.name, base64));

        result.text_edit = Some(lsp_types::TextEdit::new(range, data.name.into()).into());
    }

    fn convert_begin_environment(
        &self,
        result: &mut lsp_types::CompletionItem,
        range: lsp_types::Range,
    ) {
        if self.supports_snippets {
            result.kind = Some(Structure::Snippet.completion_kind());
            result.text_edit =
                Some(lsp_types::TextEdit::new(range, "begin{$1}\n\t\n\\end{$1}".into()).into());

            result.insert_text_format = Some(lsp_types::InsertTextFormat::SNIPPET);
        } else {
            result.kind = Some(Structure::Command.completion_kind());
            result.text_edit = Some(lsp_types::TextEdit::new(range, "begin".to_string()).into());
        }

        result.label = "begin".into();
        result.detail = Some(format_package_files(&[]));
    }

    fn convert_citation(
        &self,
        result: &mut lsp_types::CompletionItem,
        range: lsp_types::Range,
        data: completion::CitationData<'_>,
    ) {
        result.label = data.entry.name.text.clone();
        result.kind = Some(Structure::Entry(data.entry.category).completion_kind());
        result.filter_text = Some(data.entry.keywords.clone());
        let text_edit = lsp_types::TextEdit::new(range, data.entry.name.text.clone());
        result.text_edit = Some(text_edit.into());
        let resolve_info = serde_json::to_value(ResolveInfo::Citation {
            uri: data.document.uri.clone(),
            key: data.entry.name.text.clone(),
        });
        result.data = Some(resolve_info.unwrap());
    }

    fn convert_entry_type(
        &self,
        result: &mut lsp_types::CompletionItem,
        range: lsp_types::Range,
        data: EntryTypeData<'a>,
    ) {
        result.label = data.0.name.into();
        result.kind = Some(Structure::Entry(data.0.category).completion_kind());
        result.documentation = data.0.documentation.map(|value| {
            let kind = lsp_types::MarkupKind::Markdown;
            lsp_types::Documentation::MarkupContent(lsp_types::MarkupContent {
                kind,
                value: value.into(),
            })
        });

        let text_edit = lsp_types::TextEdit::new(range, data.0.name.into());
        result.text_edit = Some(text_edit.into());
    }

    fn convert_environment(
        &self,
        result: &mut lsp_types::CompletionItem,
        range: lsp_types::Range,
        data: completion::EnvironmentData<'_>,
    ) {
        result.label = data.name.into();
        result.kind = Some(Structure::Environment.completion_kind());
        result.text_edit = Some(lsp_types::TextEdit::new(range, data.name.into()).into());
        result.detail = Some(data.package.map_or_else(
            || String::from("user-defined"),
            |pkg| format_package_files(&pkg.file_names),
        ));
    }

    fn convert_glossary_entry(
        &self,
        result: &mut lsp_types::CompletionItem,
        range: lsp_types::Range,
        data: completion::GlossaryEntryData,
    ) {
        result.label = data.name.clone();
        result.kind = Some(Structure::GlossaryEntry.completion_kind());
        result.text_edit = Some(lsp_types::TextEdit::new(range, data.name).into());
    }

    fn convert_label(
        &self,
        result: &mut lsp_types::CompletionItem,
        range: lsp_types::Range,
        data: completion::LabelData<'_>,
    ) {
        let structure = match data.object {
            Some(RenderedObject::Float { .. }) => Structure::Float,
            Some(RenderedObject::Section { .. }) => Structure::Section,
            Some(RenderedObject::Theorem { .. }) => Structure::Theorem,
            Some(RenderedObject::Equation) => Structure::Equation,
            Some(RenderedObject::EnumItem) => Structure::Item,
            None => Structure::Label,
        };

        result.label = data.name.into();
        result.kind = Some(structure.completion_kind());
        result.detail = data.header;
        result.filter_text = Some(data.keywords);
        result.text_edit = Some(lsp_types::TextEdit::new(range, data.name.into()).into());
        result.documentation = data.footer.map(|footer| {
            let kind = lsp_types::MarkupKind::Markdown;
            let value = footer.into();
            lsp_types::Documentation::MarkupContent(lsp_types::MarkupContent { kind, value })
        });
    }

    fn convert_color(
        &self,
        result: &mut lsp_types::CompletionItem,
        range: lsp_types::Range,
        name: &str,
    ) {
        result.label = name.into();
        result.kind = Some(Structure::Color.completion_kind());
        result.text_edit = Some(lsp_types::TextEdit::new(range, name.into()).into());
    }

    fn convert_color_model(
        &self,
        result: &mut lsp_types::CompletionItem,
        range: lsp_types::Range,
        name: &str,
    ) {
        result.label = name.into();
        result.kind = Some(Structure::ColorModel.completion_kind());
        result.text_edit = Some(lsp_types::TextEdit::new(range, name.into()).into());
    }

    fn convert_file(
        &self,
        result: &mut lsp_types::CompletionItem,
        range: lsp_types::Range,
        name: String,
    ) {
        result.label = name.clone();
        result.kind = Some(Structure::File.completion_kind());
        result.text_edit = Some(lsp_types::TextEdit::new(range, name).into());
    }

    fn convert_directory(
        &self,
        result: &mut lsp_types::CompletionItem,
        range: lsp_types::Range,
        name: String,
    ) {
        result.label = name.clone();
        result.kind = Some(Structure::Folder.completion_kind());
        result.text_edit = Some(lsp_types::TextEdit::new(range, name).into());
    }

    fn convert_argument(
        &self,
        result: &mut lsp_types::CompletionItem,
        range: lsp_types::Range,
        data: ArgumentData<'_>,
    ) {
        result.label = data.0.name.into();
        result.kind = Some(Structure::Argument.completion_kind());
        result.text_edit = Some(lsp_types::TextEdit::new(range, data.0.name.into()).into());
        result.documentation = data
            .0
            .image
            .and_then(|base64| self.inline_image(data.0.name, base64));
    }

    fn convert_package(
        &self,
        result: &mut lsp_types::CompletionItem,
        range: lsp_types::Range,
        name: &str,
    ) {
        result.label = name.into();
        result.kind = Some(Structure::Package.completion_kind());
        result.text_edit = Some(lsp_types::TextEdit::new(range, name.into()).into());
        result.data = Some(serde_json::to_value(ResolveInfo::Package).unwrap());
    }

    fn convert_document_class(
        &self,
        result: &mut lsp_types::CompletionItem,
        range: lsp_types::Range,
        name: &str,
    ) {
        result.label = name.into();
        result.kind = Some(Structure::Class.completion_kind());
        result.text_edit = Some(lsp_types::TextEdit::new(range, name.into()).into());
        result.data = Some(serde_json::to_value(ResolveInfo::DocumentClass).unwrap());
    }

    fn convert_field(
        &self,
        result: &mut lsp_types::CompletionItem,
        range: lsp_types::Range,
        data: FieldTypeData<'a>,
    ) {
        result.label = data.0.name.into();
        result.kind = Some(Structure::Field.completion_kind());
        result.documentation = Some(lsp_types::Documentation::MarkupContent(
            lsp_types::MarkupContent {
                kind: lsp_types::MarkupKind::Markdown,
                value: data.0.documentation.into(),
            },
        ));

        let text_edit = lsp_types::TextEdit::new(range, data.0.name.into());
        result.text_edit = Some(text_edit.into());
    }

    fn convert_tikz_library(
        &self,
        result: &mut lsp_types::CompletionItem,
        range: lsp_types::Range,
        name: &str,
    ) {
        result.label = name.into();
        result.kind = Some(Structure::TikzLibrary.completion_kind());
        result.text_edit = Some(lsp_types::TextEdit::new(range, name.into()).into());
    }

    fn inline_image(&self, name: &str, base64: &str) -> Option<lsp_types::Documentation> {
        if self.supports_images {
            let kind = lsp_types::MarkupKind::Markdown;
            let value = format!("![{name}](data:image/png;base64,{base64}|width=48,height=48)");
            let content = lsp_types::MarkupContent { kind, value };
            Some(lsp_types::Documentation::MarkupContent(content))
        } else {
            None
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ResolveInfo {
    Citation { uri: lsp_types::Url, key: String },
    Package,
    DocumentClass,
}

fn format_package_files(file_names: &[&str]) -> String {
    if file_names.is_empty() {
        "built-in".into()
    } else {
        file_names.join(", ")
    }
}
