mod acronym_ref;
mod argument;
mod begin_command;
mod citation;
mod color;
mod color_model;
mod component_command;
mod component_environment;
mod entry_type;
mod field;
mod glossary_ref;
mod import;
mod include;
mod label;
mod theorem;
mod tikz_library;
mod types;
mod user_command;
mod user_environment;
mod util;

use std::borrow::Cow;

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use itertools::Itertools;
use lsp_types::{
    CompletionItem, CompletionList, CompletionParams, CompletionTextEdit, Documentation,
    InsertTextFormat, MarkupContent, MarkupKind, TextEdit,
};
use rowan::{ast::AstNode, TextSize};
use rustc_hash::FxHashSet;

use crate::{
    syntax::{bibtex, latex},
    LineIndexExt,
};

use self::{
    acronym_ref::complete_acronyms,
    argument::complete_arguments,
    begin_command::complete_begin_command,
    citation::complete_citations,
    color::complete_colors,
    color_model::complete_color_models,
    component_command::complete_component_commands,
    component_environment::complete_component_environments,
    entry_type::complete_entry_types,
    field::complete_fields,
    glossary_ref::complete_glossary_entries,
    import::complete_imports,
    include::complete_includes,
    label::complete_labels,
    theorem::complete_theorem_environments,
    tikz_library::complete_tikz_libraries,
    types::{InternalCompletionItem, InternalCompletionItemData},
    user_command::complete_user_commands,
    user_environment::complete_user_environments,
    util::{adjust_kind, component_detail, image_documentation},
};

pub use self::types::CompletionItemData;

use super::{
    cursor::{Cursor, CursorContext},
    lsp_kinds::Structure,
    FeatureRequest,
};

pub const COMPLETION_LIMIT: usize = 50;

pub fn complete(request: FeatureRequest<CompletionParams>) -> Option<CompletionList> {
    let mut items = Vec::new();
    let context = CursorContext::new(request);
    log::debug!("[Completion] Cursor: {:?}", context.cursor);
    complete_entry_types(&context, &mut items);
    complete_fields(&context, &mut items);
    complete_arguments(&context, &mut items);
    complete_citations(&context, &mut items);
    complete_imports(&context, &mut items);
    complete_colors(&context, &mut items);
    complete_color_models(&context, &mut items);
    complete_acronyms(&context, &mut items);
    complete_glossary_entries(&context, &mut items);
    complete_includes(&context, &mut items);
    complete_labels(&context, &mut items);
    complete_tikz_libraries(&context, &mut items);
    complete_component_environments(&context, &mut items);
    complete_theorem_environments(&context, &mut items);
    complete_user_environments(&context, &mut items);
    complete_begin_command(&context, &mut items);
    complete_component_commands(&context, &mut items);
    complete_user_commands(&context, &mut items);

    let mut items = dedup(items);
    preselect(&context, &mut items);
    score(&context, &mut items);

    items.sort_by_key(|item| (!item.preselect, -item.score.unwrap_or(std::i64::MIN + 1)));
    let items: Vec<_> = items
        .into_iter()
        .take(COMPLETION_LIMIT)
        .filter(|item| item.score.is_some())
        .map(|item| convert_internal_items(&context, item))
        .enumerate()
        .map(|(i, item)| append_sort_text(item, i))
        .collect();

    let is_incomplete = if context
        .request
        .context
        .client_info
        .lock()
        .unwrap()
        .as_ref()
        .map(|info| info.name.as_str())
        .unwrap_or_default()
        == "Visual Studio Code"
    {
        true
    } else {
        items.len() >= COMPLETION_LIMIT
    };

    Some(CompletionList {
        is_incomplete,
        items,
    })
}

fn dedup(items: Vec<InternalCompletionItem>) -> Vec<InternalCompletionItem> {
    let mut labels = FxHashSet::default();
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

fn score(context: &CursorContext<CompletionParams>, items: &mut Vec<InternalCompletionItem>) {
    let pattern: Cow<str> = match &context.cursor {
        Cursor::Latex(token) if token.kind().is_command_name() => {
            if token.text_range().start() + TextSize::from(1) == context.offset {
                // Handle cases similar to this one correctly:
                // $\|$ % (| is the cursor)
                "\\".into()
            } else {
                token.text().trim_end().into()
            }
        }
        Cursor::Latex(token) if token.kind() == latex::WORD => {
            if let Some(key) = token.parent().and_then(latex::Key::cast) {
                key.words()
                    .take_while(|word| word.text_range() != token.text_range())
                    .chain(std::iter::once(token.clone()))
                    .filter(|word| word.text_range().start() < context.offset)
                    .join(" ")
                    .into()
            } else {
                token.text().into()
            }
        }
        Cursor::Latex(_) => "".into(),
        Cursor::Bibtex(token) if token.kind().is_type() => token.text().into(),
        Cursor::Bibtex(token) if token.kind() == bibtex::WORD => {
            if let Some(key) = token.parent().and_then(bibtex::Key::cast) {
                key.to_string().into()
            } else {
                token.text().into()
            }
        }
        Cursor::Bibtex(token) if token.kind() == bibtex::COMMAND_NAME => {
            token.text().trim_end().into()
        }
        Cursor::Bibtex(_) => "".into(),
        Cursor::Nothing => "".into(),
    };

    let file_pattern = pattern.split('/').last().unwrap();
    let matcher = SkimMatcherV2::default().ignore_case();
    for item in items {
        item.score = match &item.data {
            InternalCompletionItemData::EntryType { ty } => {
                matcher.fuzzy_match(&ty.name, &pattern[1..])
            }
            InternalCompletionItemData::Field { field } => {
                matcher.fuzzy_match(&field.name, &pattern)
            }
            InternalCompletionItemData::Argument { name, .. } => {
                matcher.fuzzy_match(name, &pattern)
            }
            InternalCompletionItemData::BeginCommand => matcher.fuzzy_match("begin", &pattern[1..]),
            InternalCompletionItemData::Citation { key, .. } => matcher.fuzzy_match(&key, &pattern),
            InternalCompletionItemData::ComponentCommand { name, .. } => {
                matcher.fuzzy_match(name, &pattern[1..])
            }
            InternalCompletionItemData::ComponentEnvironment { name, .. } => {
                matcher.fuzzy_match(name, &pattern)
            }
            InternalCompletionItemData::Class { name } => matcher.fuzzy_match(&name, &pattern),
            InternalCompletionItemData::Package { name } => matcher.fuzzy_match(&name, &pattern),
            InternalCompletionItemData::Color { name } => matcher.fuzzy_match(&name, &pattern),
            InternalCompletionItemData::ColorModel { name } => matcher.fuzzy_match(&name, &pattern),
            InternalCompletionItemData::Acronym { name } => matcher.fuzzy_match(&name, &pattern),
            InternalCompletionItemData::GlossaryEntry { name } => {
                matcher.fuzzy_match(&name, &pattern)
            }
            InternalCompletionItemData::File { name } => matcher.fuzzy_match(&name, file_pattern),
            InternalCompletionItemData::Directory { name } => {
                matcher.fuzzy_match(&name, file_pattern)
            }
            InternalCompletionItemData::Label { name, .. } => matcher.fuzzy_match(&name, &pattern),
            InternalCompletionItemData::UserCommand { name } => {
                matcher.fuzzy_match(&name, &pattern[1..])
            }
            InternalCompletionItemData::UserEnvironment { name } => {
                matcher.fuzzy_match(&name, &pattern)
            }
            InternalCompletionItemData::PgfLibrary { name } => matcher.fuzzy_match(&name, &pattern),
            InternalCompletionItemData::TikzLibrary { name } => {
                matcher.fuzzy_match(&name, &pattern)
            }
        };
    }
}

fn preselect(
    context: &CursorContext<CompletionParams>,
    items: &mut [InternalCompletionItem],
) -> Option<()> {
    let name = context.cursor.as_latex()?;
    let group = latex::CurlyGroupWord::cast(name.parent()?)?;
    let end = latex::End::cast(group.syntax().parent()?)?;
    let environment = latex::Environment::cast(end.syntax().parent()?)?;
    let name = environment.begin()?.name()?.key()?.to_string();

    for item in items {
        if item.data.label() == name {
            item.preselect = true;
        }
    }
    Some(())
}

fn convert_internal_items(
    context: &CursorContext<CompletionParams>,
    item: InternalCompletionItem,
) -> CompletionItem {
    let range = context
        .request
        .main_document()
        .line_index
        .line_col_lsp_range(item.range);

    let mut new_item = match item.data {
        InternalCompletionItemData::EntryType { ty } => {
            let text_edit = TextEdit::new(range, (&ty.name).into());
            let kind = Structure::Entry(ty.category).completion_kind();
            CompletionItem {
                label: (&ty.name).into(),
                kind: Some(adjust_kind(&context.request, kind)),
                documentation: ty.documentation.as_ref().map(|doc| {
                    Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: doc.into(),
                    })
                }),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                data: Some(serde_json::to_value(CompletionItemData::EntryType).unwrap()),
                ..CompletionItem::default()
            }
        }
        InternalCompletionItemData::Field { field } => {
            let text_edit = TextEdit::new(range, (&field.name).into());
            CompletionItem {
                label: (&field.name).into(),
                kind: Some(adjust_kind(
                    &context.request,
                    Structure::Field.completion_kind(),
                )),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: (&field.documentation).into(),
                })),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                data: Some(serde_json::to_value(CompletionItemData::FieldName).unwrap()),
                ..CompletionItem::default()
            }
        }
        InternalCompletionItemData::Argument { name, image } => {
            let text_edit = TextEdit::new(range, name.into());
            CompletionItem {
                label: name.into(),
                kind: Some(adjust_kind(
                    &context.request,
                    Structure::Argument.completion_kind(),
                )),
                data: Some(serde_json::to_value(CompletionItemData::Argument).unwrap()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                documentation: image
                    .and_then(|image| image_documentation(&context.request, &name, image)),
                ..CompletionItem::default()
            }
        }
        InternalCompletionItemData::BeginCommand => {
            if context
                .request
                .context
                .client_capabilities
                .lock()
                .unwrap()
                .text_document
                .as_ref()
                .and_then(|cap| cap.completion.as_ref())
                .and_then(|cap| cap.completion_item.as_ref())
                .and_then(|cap| cap.snippet_support)
                == Some(true)
            {
                let text_edit = TextEdit::new(range, "begin{$1}\n\t$0\n\\end{$1}".into());
                CompletionItem {
                    kind: Some(adjust_kind(
                        &context.request,
                        Structure::Snippet.completion_kind(),
                    )),
                    data: Some(serde_json::to_value(CompletionItemData::CommandSnippet).unwrap()),
                    text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..CompletionItem::new_simple("begin".into(), component_detail(&[]))
                }
            } else {
                let text_edit = TextEdit::new(range, "begin".to_string());
                CompletionItem {
                    kind: Some(adjust_kind(
                        &context.request,
                        Structure::Command.completion_kind(),
                    )),
                    data: Some(serde_json::to_value(CompletionItemData::Command).unwrap()),
                    text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                    ..CompletionItem::new_simple("begin".to_string(), component_detail(&[]))
                }
            }
        }
        InternalCompletionItemData::Citation { uri, key, text, ty } => {
            let text_edit = TextEdit::new(range, key.to_string());
            CompletionItem {
                label: key.to_string(),
                kind: Some(adjust_kind(&context.request, ty.completion_kind())),
                filter_text: Some(text.clone()),
                sort_text: Some(text),
                data: Some(
                    serde_json::to_value(CompletionItemData::Citation {
                        uri: uri.as_ref().clone(),
                        key: key.into(),
                    })
                    .unwrap(),
                ),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::default()
            }
        }
        InternalCompletionItemData::ComponentCommand {
            name,
            image,
            glyph,
            file_names,
        } => {
            let detail = glyph.map_or_else(
                || component_detail(file_names),
                |glyph| format!("{}, {}", glyph, component_detail(file_names)),
            );
            let documentation =
                image.and_then(|img| image_documentation(&context.request, &name, img));
            let text_edit = TextEdit::new(range, name.to_string());
            CompletionItem {
                kind: Some(adjust_kind(
                    &context.request,
                    Structure::Command.completion_kind(),
                )),
                data: Some(serde_json::to_value(CompletionItemData::Command).unwrap()),
                documentation,
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::new_simple(name.to_string(), detail)
            }
        }
        InternalCompletionItemData::ComponentEnvironment { name, file_names } => {
            let text_edit = TextEdit::new(range, name.to_string());
            CompletionItem {
                kind: Some(adjust_kind(
                    &context.request,
                    Structure::Environment.completion_kind(),
                )),
                data: Some(serde_json::to_value(CompletionItemData::Environment).unwrap()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::new_simple(name.to_string(), component_detail(file_names))
            }
        }
        InternalCompletionItemData::Class { name } => {
            let text_edit = TextEdit::new(range, name.to_string());
            CompletionItem {
                label: name.into(),
                kind: Some(adjust_kind(
                    &context.request,
                    Structure::Class.completion_kind(),
                )),
                data: Some(serde_json::to_value(CompletionItemData::Class).unwrap()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::default()
            }
        }
        InternalCompletionItemData::Package { name } => {
            let text_edit = TextEdit::new(range, name.to_string());
            CompletionItem {
                label: name.into(),
                kind: Some(adjust_kind(
                    &context.request,
                    Structure::Package.completion_kind(),
                )),
                data: Some(serde_json::to_value(CompletionItemData::Package).unwrap()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::default()
            }
        }
        InternalCompletionItemData::Color { name } => {
            let text_edit = TextEdit::new(range, name.into());
            CompletionItem {
                label: name.into(),
                kind: Some(adjust_kind(
                    &context.request,
                    Structure::Color.completion_kind(),
                )),
                data: Some(serde_json::to_value(CompletionItemData::Color).unwrap()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::default()
            }
        }
        InternalCompletionItemData::ColorModel { name } => {
            let text_edit = TextEdit::new(range, name.into());
            CompletionItem {
                label: name.into(),
                kind: Some(adjust_kind(
                    &context.request,
                    Structure::ColorModel.completion_kind(),
                )),
                data: Some(serde_json::to_value(CompletionItemData::ColorModel).unwrap()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::default()
            }
        }
        InternalCompletionItemData::Acronym { name } => {
            let text_edit = TextEdit::new(range, name.to_string());
            CompletionItem {
                label: name.into(),
                kind: Some(adjust_kind(
                    &context.request,
                    Structure::GlossaryEntry.completion_kind(),
                )),
                data: Some(serde_json::to_value(CompletionItemData::Acronym).unwrap()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::default()
            }
        }
        InternalCompletionItemData::GlossaryEntry { name } => {
            let text_edit = TextEdit::new(range, name.to_string());
            CompletionItem {
                label: name.into(),
                kind: Some(adjust_kind(
                    &context.request,
                    Structure::GlossaryEntry.completion_kind(),
                )),
                data: Some(serde_json::to_value(CompletionItemData::GlossaryEntry).unwrap()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::default()
            }
        }
        InternalCompletionItemData::File { name } => {
            let text_edit = TextEdit::new(range, name.to_string());
            CompletionItem {
                label: name.into(),
                kind: Some(adjust_kind(
                    &context.request,
                    Structure::File.completion_kind(),
                )),
                data: Some(serde_json::to_value(CompletionItemData::File).unwrap()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::default()
            }
        }
        InternalCompletionItemData::Directory { name } => {
            let text_edit = TextEdit::new(range, name.to_string());
            CompletionItem {
                label: name.into(),
                kind: Some(adjust_kind(
                    &context.request,
                    Structure::Folder.completion_kind(),
                )),
                data: Some(serde_json::to_value(CompletionItemData::Folder).unwrap()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::default()
            }
        }
        InternalCompletionItemData::Label {
            name,
            kind,
            header,
            footer,
            text,
        } => {
            let text_edit = TextEdit::new(range, name.to_string());
            CompletionItem {
                label: name.into(),
                kind: Some(adjust_kind(&context.request, kind.completion_kind())),
                detail: header,
                documentation: footer.map(Documentation::String),
                sort_text: Some(text.clone()),
                filter_text: Some(text),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                data: Some(serde_json::to_value(CompletionItemData::Label).unwrap()),
                ..CompletionItem::default()
            }
        }
        InternalCompletionItemData::UserCommand { name } => {
            let detail = "user-defined".into();
            let text_edit = TextEdit::new(range, name.to_string());
            CompletionItem {
                kind: Some(adjust_kind(
                    &context.request,
                    Structure::Command.completion_kind(),
                )),
                data: Some(serde_json::to_value(CompletionItemData::Command).unwrap()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::new_simple(name.into(), detail)
            }
        }
        InternalCompletionItemData::UserEnvironment { name } => {
            let detail = "user-defined".into();
            let text_edit = TextEdit::new(range, name.to_string());
            CompletionItem {
                kind: Some(adjust_kind(
                    &context.request,
                    Structure::Environment.completion_kind(),
                )),
                data: Some(serde_json::to_value(CompletionItemData::Environment).unwrap()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::new_simple(name.into(), detail)
            }
        }
        InternalCompletionItemData::PgfLibrary { name } => {
            let text_edit = TextEdit::new(range, name.into());
            CompletionItem {
                label: name.into(),
                kind: Some(adjust_kind(
                    &context.request,
                    Structure::PgfLibrary.completion_kind(),
                )),
                data: Some(serde_json::to_value(CompletionItemData::PgfLibrary).unwrap()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                ..CompletionItem::default()
            }
        }
        InternalCompletionItemData::TikzLibrary { name } => {
            let text_edit = TextEdit::new(range, name.into());
            CompletionItem {
                label: name.into(),
                kind: Some(adjust_kind(
                    &context.request,
                    Structure::TikzLibrary.completion_kind(),
                )),
                data: Some(serde_json::to_value(CompletionItemData::TikzLibrary).unwrap()),
                text_edit: Some(CompletionTextEdit::Edit(text_edit)),
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
