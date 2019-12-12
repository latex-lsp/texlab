use futures_boxed::boxed;
use std::borrow::Cow;
use texlab_protocol::RangeExt;
use texlab_protocol::{CompletionItem, CompletionParams, Position};
use texlab_syntax::*;
use texlab_workspace::*;

pub struct OrderByQualityCompletionProvider<F> {
    pub provider: F,
}

impl<F> OrderByQualityCompletionProvider<F> {
    pub fn new(provider: F) -> Self {
        Self { provider }
    }
}

impl<F> FeatureProvider for OrderByQualityCompletionProvider<F>
where
    F: FeatureProvider<Params = CompletionParams, Output = Vec<CompletionItem>> + Send + Sync,
{
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let query = Self::get_query(
            request.document(),
            request.params.text_document_position.position,
        );
        let mut items = self.provider.execute(&request).await;
        items.sort_by_key(|item| -Self::get_quality(&query, &item));
        items
    }
}

impl<F> OrderByQualityCompletionProvider<F> {
    fn get_query(document: &Document, position: Position) -> Option<Cow<str>> {
        match &document.tree {
            SyntaxTree::Latex(tree) => {
                let node = tree
                    .find_command_by_name(position)
                    .map(LatexNode::Command)
                    .or_else(|| tree.find(position).into_iter().last())?;

                match node {
                    LatexNode::Root(_) | LatexNode::Group(_) => Some("".into()),
                    LatexNode::Command(command) => Some(command.name.text()[1..].to_owned().into()),
                    LatexNode::Text(text) => text
                        .words
                        .iter()
                        .find(|word| word.range().contains(position))
                        .map(|word| word.text().to_owned().into()),
                    LatexNode::Comma(_) => Some(",".into()),
                    LatexNode::Math(math) => Some(math.token.text().to_owned().into()),
                }
            }
            SyntaxTree::Bibtex(tree) => {
                fn get_type_query(ty: &BibtexToken, position: Position) -> Option<Cow<str>> {
                    if ty.range().contains(position) {
                        Some((&ty.text()[1..]).into())
                    } else {
                        Some("".into())
                    }
                }
                match tree.find(position).pop()? {
                    BibtexNode::Root(_) => Some("".into()),
                    BibtexNode::Preamble(preamble) => get_type_query(&preamble.ty, position),
                    BibtexNode::String(string) => get_type_query(&string.ty, position),
                    BibtexNode::Entry(entry) => get_type_query(&entry.ty, position),
                    BibtexNode::Comment(comment) => Some(comment.token.text().into()),
                    BibtexNode::Field(field) => {
                        if field.name.range().contains(position) {
                            Some(field.name.text().into())
                        } else {
                            Some("".into())
                        }
                    }
                    BibtexNode::Word(word) => Some(word.token.text().into()),
                    BibtexNode::Command(command) => Some((&command.token.text()[1..]).into()),
                    BibtexNode::QuotedContent(_)
                    | BibtexNode::BracedContent(_)
                    | BibtexNode::Concat(_) => Some("".into()),
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
