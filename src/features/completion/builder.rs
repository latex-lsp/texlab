use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use itertools::Itertools;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionList, CompletionTextEdit, Documentation,
    InsertTextFormat, MarkupContent, MarkupKind, TextEdit, Url,
};
use once_cell::sync::Lazy;
use regex::Regex;
use rowan::{ast::AstNode, TextRange, TextSize};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

use crate::{
    db::{document::Document, workspace::Workspace},
    syntax::{
        bibtex::{self, HasName, HasType},
        latex,
    },
    util::{
        capabilities::ClientCapabilitiesExt,
        cursor::{Cursor, CursorContext},
        lang_data::{BibtexEntryTypeCategory, BibtexEntryTypeDoc, BibtexFieldDoc, LANGUAGE_DATA},
        line_index_ext::LineIndexExt,
        lsp_enums::Structure,
    },
};

use super::COMPLETION_LIMIT;

pub struct CompletionBuilder<'db> {
    context: &'db CursorContext<'db>,
    items: Vec<Item<'db>>,
    matcher: SkimMatcherV2,
    text_pattern: String,
    file_pattern: String,
    preselect: Option<String>,
    snippets: bool,
    markdown: bool,
    item_kinds: &'db [CompletionItemKind],
}

impl<'db> CompletionBuilder<'db> {
    pub fn new(context: &'db CursorContext) -> Self {
        let items = Vec::new();
        let matcher = SkimMatcherV2::default().ignore_case();
        let text_pattern = match &context.cursor {
            Cursor::Tex(token) if token.kind().is_command_name() => {
                if token.text_range().start() + TextSize::from(1) == context.offset {
                    // Handle cases similar to this one correctly:
                    // $\|$ % (| is the cursor)
                    String::from("\\")
                } else {
                    token.text().trim_end().into()
                }
            }
            Cursor::Tex(token) if token.kind() == latex::WORD => {
                match token.parent().and_then(latex::Key::cast) {
                    Some(key) => key
                        .words()
                        .take_while(|word| word.text_range() != token.text_range())
                        .chain(std::iter::once(token.clone()))
                        .filter(|word| word.text_range().start() < context.offset)
                        .join(" ")
                        .into(),
                    None => token.text().into(),
                }
            }
            Cursor::Bib(token)
                if matches!(
                    token.kind(),
                    bibtex::TYPE
                        | bibtex::NAME
                        | bibtex::WORD
                        | bibtex::COMMAND_NAME
                        | bibtex::ACCENT_NAME
                ) =>
            {
                token.text().into()
            }
            Cursor::Tex(_) | Cursor::Bib(_) | Cursor::Nothing => "".into(),
        };

        let file_pattern = text_pattern.split('/').last().unwrap().to_string();

        let preselect = context
            .cursor
            .as_tex()
            .and_then(|name| name.parent())
            .and_then(latex::CurlyGroupWord::cast)
            .and_then(|group| group.syntax().parent())
            .and_then(|end| end.parent())
            .and_then(latex::Environment::cast)
            .and_then(|env| env.begin())
            .and_then(|begin| begin.name())
            .and_then(|name| name.key())
            .map(|name| name.to_string());

        let client_capabilities = context.workspace.client_capabilities(context.db);
        let snippets = client_capabilities.has_snippet_support();
        let markdown = client_capabilities.has_completion_markdown_support();
        let item_kinds = client_capabilities
            .text_document
            .as_ref()
            .and_then(|cap| cap.completion.as_ref())
            .and_then(|cap| cap.completion_item_kind.as_ref())
            .and_then(|cap| cap.value_set.as_deref())
            .unwrap_or_default();

        Self {
            context,
            items,
            matcher,
            text_pattern,
            file_pattern,
            preselect,
            snippets,
            markdown,
            item_kinds,
        }
    }

    pub fn glossary_entry(&mut self, range: TextRange, name: String) -> Option<()> {
        let score = self.matcher.fuzzy_match(&name, &self.text_pattern)?;
        self.items.push(Item {
            range,
            data: Data::GlossaryEntry { name },
            preselect: false,
            score,
        });

        Some(())
    }

    pub fn generic_argument(
        &mut self,
        range: TextRange,
        name: &'db str,
        image: Option<&'db str>,
    ) -> Option<()> {
        let score = self.matcher.fuzzy_match(name, &self.text_pattern)?;
        self.items.push(Item {
            range,
            data: Data::Argument { name, image },
            preselect: false,
            score,
        });

        Some(())
    }

    pub fn begin_snippet(&mut self, range: TextRange) -> Option<()> {
        let capabilities = Workspace::get(self.context.db).client_capabilities(self.context.db);
        if capabilities.has_snippet_support() {
            let score = self.matcher.fuzzy_match("begin", &self.text_pattern[1..])?;
            self.items.push(Item {
                range,
                data: Data::BeginSnippet,
                preselect: false,
                score,
            });
        }

        Some(())
    }

    pub fn citation(
        &mut self,
        range: TextRange,
        document: Document,
        entry: &bibtex::Entry,
    ) -> Option<()> {
        let key = entry.name_token()?.to_string();
        let score = self.matcher.fuzzy_match(&key, &self.text_pattern)?;

        let category = LANGUAGE_DATA
            .find_entry_type(&entry.type_token()?.text()[1..])
            .map_or(BibtexEntryTypeCategory::Misc, |ty| ty.category);

        let code = entry.syntax().text().to_string();
        let filter_text = format!(
            "{} {}",
            key,
            WHITESPACE_REGEX
                .replace_all(
                    &code
                        .replace('{', " ")
                        .replace('}', " ")
                        .replace(',', " ")
                        .replace('=', " "),
                    " "
                )
                .trim(),
        );

        let data = Data::Citation {
            document,
            key,
            filter_text,
            category,
        };

        self.items.push(Item {
            range,
            data,
            preselect: false,
            score,
        });

        Some(())
    }

    pub fn color_model(&mut self, range: TextRange, name: &'db str) -> Option<()> {
        let score = self.matcher.fuzzy_match(name, &self.text_pattern)?;
        self.items.push(Item {
            range,
            data: Data::ColorModel { name },
            preselect: false,
            score,
        });

        Some(())
    }

    pub fn color(&mut self, range: TextRange, name: &'db str) -> Option<()> {
        let score = self.matcher.fuzzy_match(name, &self.text_pattern)?;
        self.items.push(Item {
            range,
            data: Data::Color { name },
            preselect: false,
            score,
        });

        Some(())
    }

    pub fn component_command(
        &mut self,
        range: TextRange,
        name: &'db str,
        image: Option<&'db str>,
        glyph: Option<&'db str>,
        file_names: &'db [SmolStr],
    ) -> Option<()> {
        let score = self.matcher.fuzzy_match(name, &self.text_pattern[1..])?;
        let data = Data::ComponentCommand {
            name,
            image,
            glyph,
            file_names,
        };

        self.items.push(Item {
            range,
            data,
            preselect: false,
            score,
        });

        Some(())
    }

    pub fn component_environment(
        &mut self,
        range: TextRange,
        name: &'db str,
        file_names: &'db [SmolStr],
    ) -> Option<()> {
        let score = self.matcher.fuzzy_match(name, &self.text_pattern)?;
        self.items.push(Item {
            range,
            data: Data::ComponentEnvironment { name, file_names },
            preselect: Some(name) == self.preselect.as_deref(),
            score,
        });

        Some(())
    }

    pub fn entry_type(
        &mut self,
        range: TextRange,
        entry_type: &'db BibtexEntryTypeDoc,
    ) -> Option<()> {
        let score = self
            .matcher
            .fuzzy_match(&entry_type.name, &self.text_pattern[1..])?;

        self.items.push(Item {
            range,
            data: Data::EntryType { entry_type },
            preselect: false,
            score,
        });

        Some(())
    }

    pub fn field(&mut self, range: TextRange, field: &'db BibtexFieldDoc) -> Option<()> {
        let score = self.matcher.fuzzy_match(&field.name, &self.text_pattern)?;
        self.items.push(Item {
            range,
            data: Data::Field { field },
            preselect: false,
            score,
        });

        Some(())
    }

    pub fn class(&mut self, range: TextRange, name: &'db str) -> Option<()> {
        let score = self.matcher.fuzzy_match(name, &self.text_pattern)?;
        self.items.push(Item {
            range,
            data: Data::Class { name },
            preselect: false,
            score,
        });

        Some(())
    }

    pub fn package(&mut self, range: TextRange, name: &'db str) -> Option<()> {
        let score = self.matcher.fuzzy_match(name, &self.text_pattern)?;
        self.items.push(Item {
            range,
            data: Data::Package { name },
            preselect: false,
            score,
        });

        Some(())
    }

    pub fn file(&mut self, range: TextRange, name: String) -> Option<()> {
        let score = self.matcher.fuzzy_match(&name, &self.file_pattern)?;
        self.items.push(Item {
            range,
            data: Data::File { name },
            preselect: false,
            score,
        });

        Some(())
    }

    pub fn directory(&mut self, range: TextRange, name: String) -> Option<()> {
        let score = self.matcher.fuzzy_match(&name, &self.file_pattern)?;
        self.items.push(Item {
            range,
            data: Data::Directory { name },
            preselect: false,
            score,
        });

        Some(())
    }

    pub fn label(
        &mut self,
        range: TextRange,
        name: &'db str,
        kind: Structure,
        header: Option<String>,
        footer: Option<String>,
        text: String,
    ) -> Option<()> {
        let score = self.matcher.fuzzy_match(&name, &self.text_pattern)?;
        self.items.push(Item {
            range,
            data: Data::Label {
                name,
                kind,
                header,
                footer,
                text,
            },
            preselect: false,
            score,
        });

        Some(())
    }

    pub fn tikz_library(&mut self, range: TextRange, name: &'db str) -> Option<()> {
        let score = self.matcher.fuzzy_match(name, &self.text_pattern)?;
        self.items.push(Item {
            range,
            data: Data::TikzLibrary { name },
            preselect: false,
            score,
        });

        Some(())
    }

    pub fn user_command(&mut self, range: TextRange, name: &'db str) -> Option<()> {
        let score = self.matcher.fuzzy_match(name, &self.text_pattern[1..])?;
        self.items.push(Item {
            range,
            data: Data::UserCommand { name },
            preselect: false,
            score,
        });

        Some(())
    }

    pub fn user_environment(&mut self, range: TextRange, name: &'db str) -> Option<()> {
        let score = self.matcher.fuzzy_match(name, &self.text_pattern)?;
        self.items.push(Item {
            range,
            data: Data::UserEnvironment { name },
            preselect: Some(name) == self.preselect.as_deref(),
            score,
        });

        Some(())
    }

    pub fn finish(mut self) -> CompletionList {
        let mut list = CompletionList::default();
        list.items = std::mem::take(&mut self.items)
            .into_iter()
            .sorted_by(|a, b| {
                b.preselect
                    .cmp(&a.preselect)
                    .then_with(|| b.score.cmp(&a.score))
                    .then_with(|| a.data.label().cmp(b.data.label()))
            })
            .dedup_by(|a, b| a.data.label() == b.data.label())
            .take(COMPLETION_LIMIT)
            .enumerate()
            .map(|(i, item)| self.convert_item(item, i))
            .collect();

        list.is_incomplete = if self
            .context
            .workspace
            .client_info(self.context.db)
            .as_ref()
            .map_or(false, |client| client.name.as_str() == "Visual Studio Code")
        {
            true
        } else {
            list.items.len() >= COMPLETION_LIMIT
        };

        list
    }

    fn convert_item(&self, item: Item, index: usize) -> CompletionItem {
        let range = self.context.line_index.line_col_lsp_range(item.range);
        let preselect = item.preselect;
        let mut item = match item.data {
            Data::EntryType { entry_type } => CompletionItem {
                label: entry_type.name.clone(),
                kind: Some(Structure::Entry(entry_type.category).completion_kind()),
                documentation: entry_type.documentation.clone().map(|value| {
                    let kind = MarkupKind::Markdown;
                    Documentation::MarkupContent(MarkupContent { kind, value })
                }),
                text_edit: Some(TextEdit::new(range, entry_type.name.clone()).into()),
                ..CompletionItem::default()
            },
            Data::Field { field } => CompletionItem {
                label: field.name.clone(),
                kind: Some(Structure::Field.completion_kind()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: field.documentation.clone(),
                })),
                text_edit: Some(TextEdit::new(range, field.name.clone()).into()),
                ..CompletionItem::default()
            },
            Data::Argument { name, image } => {
                let text_edit = TextEdit::new(range, String::from(name));
                CompletionItem {
                    label: name.into(),
                    kind: Some(Structure::Argument.completion_kind()),
                    text_edit: Some(CompletionTextEdit::Edit(text_edit)),
                    documentation: image.and_then(|base64| self.inline_image(name, base64)),
                    ..CompletionItem::default()
                }
            }
            Data::BeginSnippet => {
                if self.snippets {
                    CompletionItem {
                        kind: Some(Structure::Snippet.completion_kind()),
                        text_edit: Some(
                            TextEdit::new(range, "begin{$1}\n\t\n\\end{$1}".into()).into(),
                        ),
                        insert_text_format: Some(InsertTextFormat::SNIPPET),
                        ..CompletionItem::new_simple("begin".into(), self.component_detail(&[]))
                    }
                } else {
                    CompletionItem {
                        kind: Some(Structure::Command.completion_kind()),
                        text_edit: Some(TextEdit::new(range, "begin".to_string()).into()),
                        ..CompletionItem::new_simple("begin".into(), self.component_detail(&[]))
                    }
                }
            }
            Data::Citation {
                document,
                key,
                filter_text,
                category,
            } => CompletionItem {
                label: key.clone(),
                kind: Some(Structure::Entry(category).completion_kind()),
                filter_text: Some(filter_text.clone()),
                sort_text: Some(filter_text),
                data: Some(
                    serde_json::to_value(CompletionItemData::Citation {
                        uri: document
                            .location(self.context.db)
                            .uri(self.context.db)
                            .clone(),
                        key: key.clone(),
                    })
                    .unwrap(),
                ),
                text_edit: Some(TextEdit::new(range, key).into()),
                ..CompletionItem::default()
            },
            Data::ComponentCommand {
                name,
                image,
                glyph,
                file_names,
            } => CompletionItem {
                label: name.into(),
                detail: Some(glyph.map_or_else(
                    || self.component_detail(file_names),
                    |glyph| format!("{}, {}", glyph, self.component_detail(file_names)),
                )),
                kind: Some(Structure::Command.completion_kind()),
                documentation: image.and_then(|base64| self.inline_image(name, base64)),
                text_edit: Some(TextEdit::new(range, name.into()).into()),
                ..CompletionItem::default()
            },
            Data::ComponentEnvironment { name, file_names } => CompletionItem {
                kind: Some(Structure::Environment.completion_kind()),
                text_edit: Some(TextEdit::new(range, name.into()).into()),
                ..CompletionItem::new_simple(name.into(), self.component_detail(file_names))
            },
            Data::Class { name } => CompletionItem {
                label: name.into(),
                kind: Some(Structure::Class.completion_kind()),
                data: Some(serde_json::to_value(CompletionItemData::Package).unwrap()),
                text_edit: Some(TextEdit::new(range, name.into()).into()),
                ..CompletionItem::default()
            },
            Data::Package { name } => CompletionItem {
                label: name.into(),
                kind: Some(Structure::Package.completion_kind()),
                data: Some(serde_json::to_value(CompletionItemData::Class).unwrap()),
                text_edit: Some(TextEdit::new(range, name.into()).into()),
                ..CompletionItem::default()
            },
            Data::Color { name } => CompletionItem {
                label: name.into(),
                kind: Some(Structure::Color.completion_kind()),
                text_edit: Some(TextEdit::new(range, name.into()).into()),
                ..CompletionItem::default()
            },
            Data::ColorModel { name } => CompletionItem {
                label: name.into(),
                kind: Some(Structure::ColorModel.completion_kind()),
                text_edit: Some(TextEdit::new(range, name.into()).into()),
                ..CompletionItem::default()
            },
            Data::GlossaryEntry { name } => CompletionItem {
                label: name.clone(),
                kind: Some(Structure::GlossaryEntry.completion_kind()),
                text_edit: Some(TextEdit::new(range, name).into()),
                ..CompletionItem::default()
            },
            Data::File { name } => CompletionItem {
                label: name.clone(),
                kind: Some(Structure::File.completion_kind()),
                text_edit: Some(TextEdit::new(range, name).into()),
                ..CompletionItem::default()
            },
            Data::Directory { name } => CompletionItem {
                label: name.clone(),
                kind: Some(Structure::Folder.completion_kind()),
                text_edit: Some(TextEdit::new(range, name).into()),
                ..CompletionItem::default()
            },
            Data::Label {
                name,
                kind,
                header,
                footer,
                text,
            } => CompletionItem {
                label: name.into(),
                kind: Some(kind.completion_kind()),
                detail: header,
                documentation: footer.map(Documentation::String),
                sort_text: Some(text.clone()),
                filter_text: Some(text),
                text_edit: Some(TextEdit::new(range, name.into()).into()),
                ..CompletionItem::default()
            },
            Data::UserCommand { name } => {
                let detail = "user-defined".into();
                let name = &name[1..];
                CompletionItem {
                    kind: Some(Structure::Command.completion_kind()),
                    text_edit: Some(TextEdit::new(range, name.into()).into()),
                    ..CompletionItem::new_simple(name.into(), detail)
                }
            }
            Data::UserEnvironment { name } => {
                let detail = "user-defined".into();
                CompletionItem {
                    kind: Some(Structure::Environment.completion_kind()),
                    text_edit: Some(TextEdit::new(range, name.into()).into()),
                    ..CompletionItem::new_simple(name.into(), detail)
                }
            }
            Data::TikzLibrary { name } => CompletionItem {
                label: name.into(),
                kind: Some(Structure::TikzLibrary.completion_kind()),
                text_edit: Some(TextEdit::new(range, name.into()).into()),
                ..CompletionItem::default()
            },
        };

        item.preselect = Some(preselect);

        if !self.item_kinds.contains(&item.kind.unwrap()) {
            item.kind = Some(CompletionItemKind::TEXT);
        }

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

    fn inline_image(&self, name: &str, base64: &str) -> Option<Documentation> {
        if self.markdown {
            let kind = MarkupKind::Markdown;
            let value = format!(
                "![{}](data:image/png;base64,{}|width=48,height=48)",
                name, base64
            );

            Some(Documentation::MarkupContent(MarkupContent { kind, value }))
        } else {
            None
        }
    }

    fn component_detail(&self, file_names: &[SmolStr]) -> String {
        if file_names.is_empty() {
            "built-in".into()
        } else {
            file_names.join(", ")
        }
    }
}

#[derive(Debug, Clone)]
struct Item<'db> {
    range: TextRange,
    data: Data<'db>,
    preselect: bool,
    score: i32,
}

#[derive(Debug, Clone)]
enum Data<'db> {
    EntryType {
        entry_type: &'db BibtexEntryTypeDoc,
    },
    Field {
        field: &'db BibtexFieldDoc,
    },
    Argument {
        name: &'db str,
        image: Option<&'db str>,
    },
    BeginSnippet,
    Citation {
        document: Document,
        key: String,
        filter_text: String,
        category: BibtexEntryTypeCategory,
    },
    ComponentCommand {
        name: &'db str,
        image: Option<&'db str>,
        glyph: Option<&'db str>,
        file_names: &'db [SmolStr],
    },
    ComponentEnvironment {
        name: &'db str,
        file_names: &'db [SmolStr],
    },
    Class {
        name: &'db str,
    },
    Package {
        name: &'db str,
    },
    Color {
        name: &'db str,
    },
    ColorModel {
        name: &'db str,
    },
    GlossaryEntry {
        name: String,
    },
    File {
        name: String,
    },
    Directory {
        name: String,
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
        name: &'db str,
    },
    TikzLibrary {
        name: &'db str,
    },
}

impl<'db> Data<'db> {
    pub fn label<'this: 'db>(&'this self) -> &'db str {
        match self {
            Self::EntryType { entry_type } => &entry_type.name,
            Self::Field { field } => &field.name,
            Self::Argument { name, .. } => name,
            Self::BeginSnippet => "begin",
            Self::Citation { key, .. } => key,
            Self::ComponentCommand { name, .. } => name,
            Self::ComponentEnvironment { name, .. } => name,
            Self::Class { name } => name,
            Self::Package { name } => name,
            Self::Color { name } => name,
            Self::ColorModel { name } => name,
            Self::GlossaryEntry { name } => name,
            Self::File { name } => name,
            Self::Directory { name } => name,
            Self::Label { name, .. } => name,
            Self::UserCommand { name } => name,
            Self::UserEnvironment { name } => name,
            Self::TikzLibrary { name } => name,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum CompletionItemData {
    Package,
    Class,
    Citation { uri: Url, key: String },
}

static WHITESPACE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("\\s+").unwrap());
