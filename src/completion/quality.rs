use crate::feature::FeatureRequest;
use crate::syntax::bibtex::*;
use crate::syntax::latex::*;
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use crate::workspace::Document;
use lsp_types::{CompletionItem, CompletionParams, Position};
use std::borrow::Cow;

pub struct OrderByQualityCompletionProvider;

impl OrderByQualityCompletionProvider {
    pub async fn execute<'a, E, F>(
        request: &'a FeatureRequest<CompletionParams>,
        execute: E,
    ) -> Vec<CompletionItem>
    where
        E: Fn(&'a FeatureRequest<CompletionParams>) -> F,
        F: std::future::Future<Output = Vec<CompletionItem>>,
    {
        let query = Self::get_query(&request.document, request.params.position);
        let mut items = await!(execute(&request));
        items.sort_by_key(|item| -Self::get_quality(&query, &item.label));
        items
    }

    fn get_query(document: &Document, position: Position) -> Option<Cow<str>> {
        match &document.tree {
            SyntaxTree::Latex(tree) => {
                let node = tree
                    .find_command(position)
                    .map(LatexNode::Command)
                    .or_else(|| tree.find(position).into_iter().last())?;

                match node {
                    LatexNode::Root(_) | LatexNode::Group(_) => Some(Cow::from("")),
                    LatexNode::Command(command) => {
                        Some(Cow::from(command.name.text()[1..].to_owned()))
                    }
                    LatexNode::Text(text) => {
                        text.words.last().map(|w| Cow::from(w.text().to_owned()))
                    }
                }
            }
            SyntaxTree::Bibtex(tree) => {
                fn get_type_query(ty: &BibtexToken, position: Position) -> Option<Cow<str>> {
                    if ty.range().contains(position) {
                        Some(Cow::from(&ty.text()[1..]))
                    } else {
                        Some(Cow::from(""))
                    }
                }
                let mut finder = BibtexFinder::new(position);
                finder.visit_root(&tree.root);
                match finder.results.pop()? {
                    BibtexNode::Root(_) => Some(Cow::from("")),
                    BibtexNode::Preamble(preamble) => get_type_query(&preamble.ty, position),
                    BibtexNode::String(string) => get_type_query(&string.ty, position),
                    BibtexNode::Entry(entry) => get_type_query(&entry.ty, position),
                    BibtexNode::Comment(comment) => Some(Cow::from(comment.token.text())),
                    BibtexNode::Field(field) => {
                        if field.name.range().contains(position) {
                            Some(Cow::from(field.name.text()))
                        } else {
                            Some(Cow::from(""))
                        }
                    }
                    BibtexNode::Word(word) => Some(Cow::from(word.token.text())),
                    BibtexNode::Command(command) => Some(Cow::from(&command.token.text()[1..])),
                    BibtexNode::QuotedContent(_)
                    | BibtexNode::BracedContent(_)
                    | BibtexNode::Concat(_) => Some(Cow::from("")),
                }
            }
        }
    }

    fn get_quality(query: &Option<Cow<str>>, label: &str) -> i32 {
        if let Some(query) = query {
            if label == query {
                return 7;
            }

            if label.to_lowercase() == query.to_lowercase() {
                return 6;
            }

            if label.starts_with(query.as_ref()) {
                return 5;
            }

            if label.to_lowercase().starts_with(&query.to_lowercase()) {
                return 4;
            }

            if label.contains(query.as_ref()) {
                return 3;
            }

            if label.to_lowercase().contains(&query.to_lowercase()) {
                return 2;
            }

            return 1;
        } else {
            return 0;
        }
    }
}
