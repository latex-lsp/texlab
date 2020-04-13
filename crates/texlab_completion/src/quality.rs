use futures_boxed::boxed;
use std::borrow::Cow;
use texlab_feature::{Document, DocumentContent, FeatureProvider, FeatureRequest};
use texlab_protocol::{CompletionItem, CompletionParams, Position, RangeExt};
use texlab_syntax::{bibtex, latex, SyntaxNode};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct OrderByQualityCompletionProvider<F>(pub F);

impl<F> FeatureProvider for OrderByQualityCompletionProvider<F>
where
    F: FeatureProvider<Params = CompletionParams, Output = Vec<CompletionItem>> + Send + Sync,
{
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let query = Self::query(req.current(), req.params.text_document_position.position);
        let mut items = self.0.execute(req).await;
        items.sort_by_key(|item| -Self::get_quality(&query, &item));
        items
    }
}

impl<F> OrderByQualityCompletionProvider<F> {
    fn query(doc: &Document, pos: Position) -> Option<Cow<str>> {
        match &doc.content {
            DocumentContent::Latex(table) => {
                fn command_query(cmd: &latex::Command) -> Cow<str> {
                    cmd.name.text()[1..].to_owned().into()
                }

                if let Some(node) = table.find_command_by_short_name_range(pos) {
                    return Some(command_query(table.as_command(node).unwrap()));
                }

                match &table[table.find(pos).into_iter().last()?] {
                    latex::Node::Root(_) | latex::Node::Group(_) => Some("".into()),
                    latex::Node::Command(cmd) => Some(command_query(cmd)),
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

    fn get_quality(query: &Option<Cow<str>>, item: &CompletionItem) -> i32 {
        if item.preselect == Some(true) {
            return 8;
        }

        let label = &item.label;
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

            1
        } else {
            0
        }
    }
}
