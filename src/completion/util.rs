use crate::{
    feature::FeatureRequest,
    protocol::{
        CompletionItemKind, CompletionParams, Documentation, MarkupContent, MarkupKind, Position,
        RangeExt,
    },
    syntax::{bibtex, latex, SyntaxNode},
    workspace::DocumentContent,
};
use std::borrow::Cow;

pub fn current_word(req: &FeatureRequest<CompletionParams>) -> Option<Cow<str>> {
    let pos = req.params.text_document_position.position;
    match &req.current().content {
        DocumentContent::Latex(table) => {
            if let Some(node) = table.find_command_by_short_name_range(pos) {
                return Some(command_word(table.as_command(node).unwrap()));
            }

            match &table[table.find(pos).into_iter().last()?] {
                latex::Node::Root(_) | latex::Node::Group(_) => Some("".into()),
                latex::Node::Command(cmd) => Some(command_word(cmd)),
                latex::Node::Text(text) => text
                    .words
                    .iter()
                    .find(|word| word.range().contains(pos))
                    .map(|word| word.text().split('/').last().unwrap().to_owned().into()),
                latex::Node::Comma(_) => Some(",".into()),
                latex::Node::Math(math) => Some(math.token.text().to_owned().into()),
            }
        }
        DocumentContent::Bibtex(tree) => {
            fn type_query(ty: &bibtex::Token, pos: Position) -> Option<Cow<str>> {
                if ty.range().contains(pos) {
                    Some((&ty.text()[1..]).into())
                } else {
                    Some("".into())
                }
            }

            match &tree.graph[tree.find(pos).pop()?] {
                bibtex::Node::Root(_) => Some("".into()),
                bibtex::Node::Preamble(preamble) => type_query(&preamble.ty, pos),
                bibtex::Node::String(string) => type_query(&string.ty, pos),
                bibtex::Node::Entry(entry) => type_query(&entry.ty, pos),
                bibtex::Node::Comment(comment) => Some(comment.token.text().into()),
                bibtex::Node::Field(field) => {
                    if field.name.range().contains(pos) {
                        Some(field.name.text().into())
                    } else {
                        Some("".into())
                    }
                }
                bibtex::Node::Word(word) => Some(word.token.text().into()),
                bibtex::Node::Command(cmd) => Some((&cmd.token.text()[1..]).into()),
                bibtex::Node::QuotedContent(_)
                | bibtex::Node::BracedContent(_)
                | bibtex::Node::Concat(_) => Some("".into()),
            }
        }
    }
}

fn command_word(cmd: &latex::Command) -> Cow<str> {
    cmd.name.text()[1..].into()
}

pub fn component_detail(file_names: &[String]) -> String {
    if file_names.is_empty() {
        "built-in".to_owned()
    } else {
        file_names.join(", ")
    }
}

pub fn image_documentation(
    req: &FeatureRequest<CompletionParams>,
    name: &str,
    image: &str,
) -> Option<Documentation> {
    if supports_images(req) {
        Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!(
                "![{}](data:image/png;base64,{}|width=48,height=48)",
                name, image
            ),
        }))
    } else {
        None
    }
}

fn supports_images(req: &FeatureRequest<CompletionParams>) -> bool {
    req.client_capabilities
        .text_document
        .as_ref()
        .and_then(|cap| cap.completion.as_ref())
        .and_then(|cap| cap.completion_item.as_ref())
        .and_then(|cap| cap.documentation_format.as_ref())
        .map_or(true, |formats| formats.contains(&MarkupKind::Markdown))
}

pub fn adjust_kind(
    req: &FeatureRequest<CompletionParams>,
    kind: CompletionItemKind,
) -> CompletionItemKind {
    if let Some(value_set) = req
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
