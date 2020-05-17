use crate::{
    feature::{FeatureProvider, FeatureRequest},
    protocol::{CompletionItem, CompletionParams, Position, RangeExt},
    syntax::{bibtex, latex, SyntaxNode},
    workspace::{Document, DocumentContent},
};
use async_trait::async_trait;
use std::borrow::Cow;

#[derive(Debug)]
pub struct QualityEvaluator<'a> {
    query: Option<Cow<'a, str>>,
}

impl<'a> QualityEvaluator<'a> {
    pub fn quality_of(&self, label: &str, preselect: &Option<bool>) -> i32 {
        if *preselect == Some(true) {
            return 8;
        }

        if let Some(query) = &self.query {
            if label == query {
                return 7;
            }

            let label_ci = label.to_lowercase();
            let query_ci = query.to_lowercase();
            if label_ci == query_ci {
                return 6;
            }

            if label.starts_with(query.as_ref()) {
                return 5;
            }

            if label_ci.starts_with(&query_ci) {
                return 4;
            }

            if label.contains(query.as_ref()) {
                return 3;
            }

            if label_ci.contains(&query_ci) {
                return 2;
            }

            1
        } else {
            0
        }
    }

    pub fn parse(doc: &'a Document, pos: Position) -> Self {
        Self {
            query: Self::parse_query(doc, pos),
        }
    }

    fn parse_query(doc: &'a Document, pos: Position) -> Option<Cow<'a, str>> {
        match &doc.content {
            DocumentContent::Latex(table) => {
                if let Some(node) = table.find_command_by_short_name_range(pos) {
                    return Some(Self::command_query(table.as_command(node).unwrap()));
                }

                match &table[table.find(pos).into_iter().last()?] {
                    latex::Node::Root(_) | latex::Node::Group(_) => Some("".into()),
                    latex::Node::Command(cmd) => Some(Self::command_query(cmd)),
                    latex::Node::Text(text) => text
                        .words
                        .iter()
                        .find(|word| word.range().contains(pos))
                        .map(|word| word.text().to_owned().into()),
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

    fn command_query(cmd: &'a latex::Command) -> Cow<'a, str> {
        cmd.name.text()[1..].to_owned().into()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct OrderByQualityCompletionProvider<F>(pub F);

#[async_trait]
impl<F> FeatureProvider for OrderByQualityCompletionProvider<F>
where
    F: FeatureProvider<Params = CompletionParams, Output = Vec<CompletionItem>> + Send + Sync,
{
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let pos = req.params.text_document_position.position;
        let eval = QualityEvaluator::parse(req.current(), pos);
        let mut items: Vec<_> = self
            .0
            .execute(req)
            .await
            .into_iter()
            .map(|inner| QualityCompletionItem {
                quality: -eval.quality_of(&inner.label, &inner.preselect),
                inner,
            })
            .filter(|item| -item.quality > 1)
            .collect();

        items.sort_by_key(|item| item.quality);
        items.into_iter().map(|item| item.inner).collect()
    }
}

struct QualityCompletionItem {
    inner: CompletionItem,
    quality: i32,
}
