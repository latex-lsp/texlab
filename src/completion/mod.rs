mod bibtex;
mod latex;
mod types;
mod util;

pub use self::types::{CompletionItemData, Item, ItemData};

use self::{
    bibtex::{
        cmd::complete_bibtex_commands, entry_type::complete_bibtex_entry_types,
        field_name::complete_bibtex_fields,
    },
    latex::{
        argument::complete_latex_arguments,
        begin_cmd::complete_latex_begin_command,
        citation::complete_latex_citations,
        color::complete_latex_colors,
        color_model::complete_latex_color_models,
        component::{complete_latex_component_commands, complete_latex_component_environments},
        glossary::complete_latex_glossary_entries,
        import::{complete_latex_classes, complete_latex_packages},
        include::complete_latex_includes,
        label::complete_latex_labels,
        theorem::complete_latex_theorem_environments,
        tikz_lib::{complete_latex_pgf_libraries, complete_latex_tikz_libraries},
        user::{complete_latex_user_commands, complete_latex_user_environments},
    },
    util::{adjust_kind, component_detail, current_word, image_documentation},
};
use crate::{
    feature::{FeatureProvider, FeatureRequest},
    protocol::{
        CompletionItem, CompletionParams, CompletionTextEdit, Documentation, InsertTextFormat,
        MarkupContent, MarkupKind, RangeExt, TextEdit,
    },
    syntax::{self, Structure, SyntaxNode},
    workspace::DocumentContent,
};
use async_trait::async_trait;
use fuzzy_matcher::skim::fuzzy_match;
use std::collections::HashSet;

pub const COMPLETION_LIMIT: usize = 50;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct CompletionProvider;

impl CompletionProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl FeatureProvider for CompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let items = complete_all(req).await;
        let mut items = dedup(items);
        preselect(req, &mut items);
        score(req, &mut items);

        items.sort_by_key(|item| (!item.preselect, -item.score.unwrap_or(std::i64::MIN + 1)));
        items
            .into_iter()
            .take(COMPLETION_LIMIT)
            .filter(|item| item.score.is_some())
            .map(|item| convert(req, item))
            .enumerate()
            .map(|(i, item)| append_sort_text(item, i))
            .collect()
    }
}

async fn complete_all<'a>(req: &'a FeatureRequest<CompletionParams>) -> Vec<Item<'a>> {
    let mut items = Vec::new();
    complete_bibtex_commands(req, &mut items).await;
    complete_bibtex_entry_types(req, &mut items).await;
    complete_bibtex_fields(req, &mut items).await;
    complete_latex_arguments(req, &mut items).await;
    complete_latex_begin_command(req, &mut items).await;
    complete_latex_colors(req, &mut items).await;
    complete_latex_color_models(req, &mut items).await;
    complete_latex_glossary_entries(req, &mut items).await;
    complete_latex_citations(req, &mut items).await;
    complete_latex_classes(req, &mut items).await;
    complete_latex_packages(req, &mut items).await;
    complete_latex_includes(req, &mut items).await;
    complete_latex_labels(req, &mut items).await;
    complete_latex_pgf_libraries(req, &mut items).await;
    complete_latex_tikz_libraries(req, &mut items).await;
    complete_latex_component_environments(req, &mut items).await;
    complete_latex_theorem_environments(req, &mut items).await;
    complete_latex_user_environments(req, &mut items).await;
    complete_latex_component_commands(req, &mut items).await;
    complete_latex_user_commands(req, &mut items).await;
    items
}

fn dedup<'a>(items: Vec<Item<'a>>) -> Vec<Item<'a>> {
    let mut labels = HashSet::new();
    let mut insert = vec![false; items.len()];
    for (i, item) in items.iter().enumerate() {
        insert[i] = labels.insert(item.data.label());
    }
    items
        .into_iter()
        .enumerate()
        .filter(|(i, _)| insert[*i])
        .map(|(_, item)| item)
        .collect()
}

fn preselect(req: &FeatureRequest<CompletionParams>, items: &mut [Item]) {
    let pos = req.params.text_document_position.position;
    if let DocumentContent::Latex(table) = &req.current().content {
        for env in &table.environments {
            if let Some(name) = env.left.name(&table) {
                let right_args = table
                    .extract_group(env.right.parent, syntax::latex::GroupKind::Group, 0)
                    .unwrap();
                let right_args_range = table[right_args].range();
                let cond1 = right_args_range.contains_exclusive(pos);
                let cond2 = table
                    .as_group(right_args)
                    .and_then(|group| group.right.as_ref())
                    .is_none()
                    && right_args_range.contains(pos);

                if cond1 || cond2 {
                    for symbol in items.iter_mut() {
                        if symbol.data.label() == name.text() {
                            symbol.preselect = true;
                            break;
                        }
                    }
                }
            }
        }
    }
}

fn score(req: &FeatureRequest<CompletionParams>, items: &mut Vec<Item>) {
    let current_word = current_word(req);
    let pattern = current_word.as_deref().unwrap_or_default();
    for item in items {
        item.score = match &item.data {
            ItemData::ComponentCommand { name, .. } => fuzzy_match(name, pattern),
            ItemData::ComponentEnvironment { name, .. } => fuzzy_match(name, pattern),
            ItemData::UserCommand { name } => fuzzy_match(name, pattern),
            ItemData::UserEnvironment { name } => fuzzy_match(name, pattern),
            ItemData::Label { text, .. } => fuzzy_match(&text, pattern),
            ItemData::Class { name } => fuzzy_match(&name, pattern),
            ItemData::Package { name } => fuzzy_match(&name, pattern),
            ItemData::PgfLibrary { name } => fuzzy_match(name, pattern),
            ItemData::TikzLibrary { name } => fuzzy_match(name, pattern),
            ItemData::File { name } => fuzzy_match(name, pattern),
            ItemData::Directory { name } => fuzzy_match(name, pattern),
            ItemData::Citation { text, .. } => fuzzy_match(&text, pattern),
            ItemData::Argument { name, .. } => fuzzy_match(&name, pattern),
            ItemData::BeginCommand => fuzzy_match("begin", pattern),
            ItemData::Color { name } => fuzzy_match(name, pattern),
            ItemData::ColorModel { name } => fuzzy_match(name, pattern),
            ItemData::GlossaryEntry { name } => fuzzy_match(name, pattern),
            ItemData::EntryType { ty } => fuzzy_match(&ty.name, pattern),
            ItemData::Field { field } => fuzzy_match(&field.name, pattern),
        };
    }
}

fn convert(req: &FeatureRequest<CompletionParams>, item: Item) -> CompletionItem {
    let mut new_item = match item.data {
        ItemData::ComponentCommand {
            name,
            image,
            glyph,
            file_names,
        } => {
            let detail = glyph.map_or_else(
                || component_detail(file_names),
                |glyph| format!("{}, {}", glyph, component_detail(file_names)),
            );
            let documentation = image.and_then(|img| image_documentation(&req, &name, img));
            let text_edit = TextEdit::new(item.range, name.into());
            CompletionItem {
                kind: Some(adjust_kind(req, Structure::Command.completion_kind())),
                data: Some(CompletionItemData::Command.into()),
                documentation,
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::new_simple(name.into(), detail)
            }
        }
        ItemData::ComponentEnvironment { name, file_names } => {
            let text_edit = TextEdit::new(item.range, name.into());
            CompletionItem {
                kind: Some(adjust_kind(req, Structure::Environment.completion_kind())),
                data: Some(CompletionItemData::Environment.into()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::new_simple(name.into(), component_detail(file_names))
            }
        }
        ItemData::UserCommand { name } => {
            let detail = "user-defined".into();
            let text_edit = TextEdit::new(item.range, name.into());
            CompletionItem {
                kind: Some(adjust_kind(req, Structure::Command.completion_kind())),
                data: Some(CompletionItemData::Command.into()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::new_simple(name.into(), detail)
            }
        }
        ItemData::UserEnvironment { name } => {
            let detail = "user-defined".into();
            let text_edit = TextEdit::new(item.range, name.into());
            CompletionItem {
                kind: Some(adjust_kind(req, Structure::Environment.completion_kind())),
                data: Some(CompletionItemData::Environment.into()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::new_simple(name.into(), detail)
            }
        }
        ItemData::Label {
            name,
            kind,
            header,
            footer,
            text,
        } => {
            let text_edit = TextEdit::new(item.range, name.into());
            CompletionItem {
                label: name.into(),
                kind: Some(adjust_kind(req, kind.completion_kind())),
                data: Some(CompletionItemData::Label.into()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                filter_text: Some(text.clone()),
                sort_text: Some(text),
                detail: header,
                documentation: footer.map(Documentation::String),
                ..CompletionItem::default()
            }
        }
        ItemData::Class { name } => {
            let text_edit = TextEdit::new(item.range, name.as_ref().into());
            CompletionItem {
                label: name.into_owned(),
                kind: Some(adjust_kind(req, Structure::Class.completion_kind())),
                data: Some(CompletionItemData::Class.into()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::default()
            }
        }
        ItemData::Package { name } => {
            let text_edit = TextEdit::new(item.range, name.as_ref().into());
            CompletionItem {
                label: name.into_owned(),
                kind: Some(adjust_kind(req, Structure::Package.completion_kind())),
                data: Some(CompletionItemData::Package.into()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::default()
            }
        }
        ItemData::PgfLibrary { name } => {
            let text_edit = TextEdit::new(item.range, name.into());
            CompletionItem {
                label: name.into(),
                kind: Some(adjust_kind(req, Structure::PgfLibrary.completion_kind())),
                data: Some(CompletionItemData::PgfLibrary.into()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::default()
            }
        }
        ItemData::TikzLibrary { name } => {
            let text_edit = TextEdit::new(item.range, name.into());
            CompletionItem {
                label: name.into(),
                kind: Some(adjust_kind(req, Structure::TikzLibrary.completion_kind())),
                data: Some(CompletionItemData::TikzLibrary.into()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::default()
            }
        }
        ItemData::File { name } => {
            let text_edit = TextEdit::new(item.range, name.clone());
            CompletionItem {
                label: name,
                kind: Some(adjust_kind(req, Structure::File.completion_kind())),
                data: Some(CompletionItemData::File.into()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::default()
            }
        }
        ItemData::Directory { name } => {
            let text_edit = TextEdit::new(item.range, name.clone());
            CompletionItem {
                label: name,
                kind: Some(adjust_kind(req, Structure::Folder.completion_kind())),
                data: Some(CompletionItemData::Folder.into()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::default()
            }
        }
        ItemData::Citation { uri, key, text, ty } => {
            let text_edit = TextEdit::new(item.range, key.into());
            CompletionItem {
                label: key.into(),
                kind: Some(adjust_kind(req, ty.completion_kind())),
                filter_text: Some(text.clone()),
                sort_text: Some(text),
                data: Some(
                    CompletionItemData::Citation {
                        uri: uri.clone(),
                        key: key.into(),
                    }
                    .into(),
                ),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::default()
            }
        }
        ItemData::Argument { name, image } => {
            let text_edit = TextEdit::new(item.range, name.into());
            CompletionItem {
                label: name.into(),
                kind: Some(adjust_kind(req, Structure::Argument.completion_kind())),
                data: Some(CompletionItemData::Argument.into()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                documentation: image.and_then(|image| image_documentation(&req, &name, image)),
                ..CompletionItem::default()
            }
        }
        ItemData::BeginCommand => CompletionItem {
            kind: Some(adjust_kind(req, Structure::Snippet.completion_kind())),
            data: Some(CompletionItemData::CommandSnippet.into()),
            insert_text: Some("begin{$1}\n\t$0\n\\end{$1}".into()),
            insert_text_format: Some(InsertTextFormat::Snippet),
            ..CompletionItem::new_simple("begin".into(), component_detail(&[]))
        },
        ItemData::Color { name } => {
            let text_edit = TextEdit::new(item.range, name.into());
            CompletionItem {
                label: name.into(),
                kind: Some(adjust_kind(req, Structure::Color.completion_kind())),
                data: Some(CompletionItemData::Color.into()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::default()
            }
        }
        ItemData::ColorModel { name } => {
            let text_edit = TextEdit::new(item.range, name.into());
            CompletionItem {
                label: name.into(),
                kind: Some(adjust_kind(req, Structure::ColorModel.completion_kind())),
                data: Some(CompletionItemData::ColorModel.into()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::default()
            }
        }
        ItemData::GlossaryEntry { name } => {
            let text_edit = TextEdit::new(item.range, name.into());
            CompletionItem {
                label: name.into(),
                kind: Some(adjust_kind(req, Structure::GlossaryEntry.completion_kind())),
                data: Some(CompletionItemData::GlossaryEntry.into()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::default()
            }
        }
        ItemData::EntryType { ty } => {
            let text_edit = TextEdit::new(item.range, (&ty.name).into());
            let kind = Structure::Entry(ty.category).completion_kind();
            CompletionItem {
                label: (&ty.name).into(),
                kind: Some(adjust_kind(req, kind)),
                data: Some(CompletionItemData::EntryType.into()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                documentation: ty.documentation.as_ref().map(|doc| {
                    Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: doc.into(),
                    })
                }),
                ..CompletionItem::default()
            }
        }
        ItemData::Field { field } => {
            let text_edit = TextEdit::new(item.range, (&field.name).into());
            CompletionItem {
                label: (&field.name).into(),
                kind: Some(adjust_kind(req, Structure::Field.completion_kind())),
                data: Some(CompletionItemData::FieldName.into()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: (&field.documentation).into(),
                })),
                ..CompletionItem::default()
            }
        }
    };
    new_item.preselect = Some(item.preselect);
    new_item
}

fn append_sort_text(mut item: CompletionItem, index: usize) -> CompletionItem {
    let sort_prefix = format!("{:0>2}", index);
    match &item.sort_text {
        Some(sort_text) => {
            item.sort_text = Some(format!("{} {}", sort_prefix, sort_text));
        }
        None => {
            item.sort_text = Some(sort_prefix);
        }
    };
    item
}
