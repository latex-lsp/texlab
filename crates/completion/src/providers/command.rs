use base_db::{semantics::Span, DocumentData};
use completion_data::included_packages;
use rowan::{TextRange, TextSize};
use syntax::{bibtex, latex};

use crate::{
    util::{CompletionBuilder, ProviderContext},
    CommandData, CompletionItem, CompletionItemData, CompletionParams,
};

static DELIMITERS: &[(&str, &str)] = &[("(", ")"), ("[", "]"), ("{", "\\}")];

pub fn complete_commands<'a>(
    params: &'a CompletionParams<'a>,
    builder: &mut CompletionBuilder<'a>,
) -> Option<()> {
    let cursor = find_command_name(params)?;

    let mut proc = Processor(ProviderContext {
        builder,
        params,
        cursor,
    });

    proc.add_begin_snippet();
    proc.add_delimiters();
    proc.add_library();
    proc.add_user();
    Some(())
}

struct Processor<'a, 'b>(ProviderContext<'a, 'b>);

impl<'a, 'b> Processor<'a, 'b> {
    pub fn add_begin_snippet(&mut self) -> Option<()> {
        let score = self.0.builder.matcher.score("begin", &self.0.cursor.text)?;
        let data = CompletionItemData::BeginEnvironment;
        self.0
            .builder
            .items
            .push(CompletionItem::new_simple(score, self.0.cursor.range, data));

        Some(())
    }

    pub fn add_delimiters(&mut self) {
        for (left, right) in DELIMITERS {
            let Some(score) = self.0.builder.matcher.score(left, &self.0.cursor.text) else {
                continue;
            };

            let data = CompletionItemData::CommandLikeDelimiter(left, right);
            self.0
                .builder
                .items
                .push(CompletionItem::new_simple(score, self.0.cursor.range, data));
        }
    }

    pub fn add_library(&mut self) -> Option<()> {
        for package in included_packages(&self.0.params.feature) {
            let commands_with_score = package.commands.iter().filter_map(|command| {
                let matcher = &self.0.builder.matcher;
                let score = matcher.score(&command.name, &self.0.cursor.text)?;
                Some((command, score))
            });

            for (command, score) in commands_with_score {
                let data = CompletionItemData::Command(CommandData {
                    name: &command.name,
                    glyph: command.glyph.as_deref(),
                    image: command.image,
                    package: Some(package),
                });

                self.0.builder.items.push(CompletionItem::new_simple(
                    score,
                    self.0.cursor.range,
                    data,
                ));
            }
        }

        Some(())
    }

    fn add_user(&mut self) {
        let documents = self.0.params.feature.project.documents.iter();
        for data in documents.filter_map(|document| document.data.as_tex()) {
            let commands = data
                .semantics
                .commands
                .iter()
                .filter(|name| name.range != self.0.cursor.range);

            let commands_with_score = commands.filter_map(|command| {
                let matcher = &self.0.builder.matcher;
                let score = matcher.score(&command.text, &self.0.cursor.text)?;
                Some((command, score))
            });

            for (command, score) in commands_with_score {
                let data = CompletionItemData::Command(CommandData {
                    name: &command.text,
                    glyph: None,
                    image: None,
                    package: None,
                });

                self.0.builder.items.push(CompletionItem::new_simple(
                    score,
                    self.0.cursor.range,
                    data,
                ));
            }
        }
    }
}

fn find_command_name(params: &CompletionParams) -> Option<Span> {
    let offset = params.offset;
    match &params.feature.document.data {
        DocumentData::Tex(data) => {
            let root = data.root_node();
            find_command_name_ast(&root, latex::COMMAND_NAME, offset)
        }
        DocumentData::Bib(data) => {
            let root = data.root_node();
            find_command_name_ast(&root, bibtex::COMMAND_NAME, offset)
                .or_else(|| find_command_name_ast(&root, bibtex::ACCENT_NAME, offset))
        }
        _ => None,
    }
}

fn find_command_name_ast<L: rowan::Language>(
    root: &rowan::SyntaxNode<L>,
    kind: L::Kind,
    offset: TextSize,
) -> Option<Span> {
    // `token_at_offset` will panic if offset is not within the range of `root`.
    if !root.text_range().contains(offset) {
        return None;
    }
    let token = root
        .token_at_offset(offset)
        .filter(|token| token.text_range().start() != offset)
        .find(|token| token.kind() == kind)?;

    let full_range = token.text_range();
    let text = String::from(&token.text()[1..]);
    Some(Span::new(
        text,
        TextRange::new(full_range.start() + TextSize::of('\\'), full_range.end()),
    ))
}
