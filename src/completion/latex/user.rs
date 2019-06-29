use crate::completion::factory::{self, LatexComponentId};
use crate::completion::latex::combinators;
use crate::feature::{FeatureProvider, FeatureRequest};
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use futures_boxed::boxed;
use itertools::Itertools;
use lsp_types::{CompletionItem, CompletionParams, TextEdit};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexUserCommandCompletionProvider;

impl FeatureProvider for LatexUserCommandCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        combinators::command(request, async move |current_command| {
            let mut items = Vec::new();
            for document in request.related_documents() {
                if let SyntaxTree::Latex(tree) = &document.tree {
                    tree.commands
                        .iter()
                        .filter(|command| command.range() != current_command.range())
                        .map(|command| &command.name.text()[1..])
                        .unique()
                        .map(|command| {
                            let text_edit = TextEdit::new(
                                current_command.short_name_range(),
                                command.to_owned().into(),
                            );
                            factory::command(
                                request,
                                command.to_owned().into(),
                                text_edit,
                                &LatexComponentId::User,
                            )
                        })
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
