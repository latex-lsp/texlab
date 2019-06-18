use crate::completion::factory;
use crate::completion::factory::LatexComponentId;
use crate::completion::latex::combinators::LatexCombinators;
use crate::feature::{FeatureProvider, FeatureRequest};
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use futures_boxed::boxed;
use itertools::Itertools;
use lsp_types::{CompletionItem, CompletionParams};
use std::borrow::Cow;
use std::sync::Arc;

pub struct LatexUserCommandCompletionProvider;

impl FeatureProvider for LatexUserCommandCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<Arc<CompletionItem>>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        LatexCombinators::command(request, async move |current_command| {
            let mut items = Vec::new();
            for document in request.related_documents() {
                if let SyntaxTree::Latex(tree) = &document.tree {
                    tree.commands
                        .iter()
                        .filter(|command| command.range() != current_command.range())
                        .map(|command| &command.name.text()[1..])
                        .unique()
                        .map(|name| Cow::from(name.to_owned()))
                        .map(|name| factory::create_command(name, &LatexComponentId::Unknown))
                        .map(Arc::new)
                        .for_each(|item| items.push(item));
                }
            }
            items
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{test_feature, FeatureSpec};
    use lsp_types::Position;

    #[test]
    fn test() {
        let items = test_feature(
            LatexUserCommandCompletionProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\include{bar.tex}\n\\foo"),
                    FeatureSpec::file("bar.tex", "\\bar"),
                    FeatureSpec::file("baz.tex", "\\baz"),
                ],
                main_file: "foo.tex",
                position: Position::new(1, 2),
                ..FeatureSpec::default()
            },
        );
        let labels: Vec<&str> = items.iter().map(|item| item.label.as_ref()).collect();
        assert_eq!(labels, vec!["include", "bar"]);
    }
}
