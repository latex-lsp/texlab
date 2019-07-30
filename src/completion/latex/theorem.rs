use crate::completion::factory::{self, LatexComponentId};
use crate::completion::latex::combinators;
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams, TextEdit};
use texlab_syntax::*;
use texlab_workspace::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexTheoremEnvironmentCompletionProvider;

impl FeatureProvider for LatexTheoremEnvironmentCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        combinators::environment(request, async move |context| {
            let mut items = Vec::new();
            for document in request.related_documents() {
                if let SyntaxTree::Latex(tree) = &document.tree {
                    for theorem in &tree.theorem_definitions {
                        let name = theorem.name().text().to_owned();
                        let text_edit = TextEdit::new(context.range, name.clone().into());
                        let item = factory::environment(
                            request,
                            name.into(),
                            text_edit,
                            &LatexComponentId::User,
                        );
                        items.push(item);
                    }
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
    use lsp_types::{Position, Range};
    use std::borrow::Cow;

    #[test]
    fn test() {
        let items = test_feature(
            LatexTheoremEnvironmentCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.tex",
                    "\\newtheorem{theorem}{Theorem}\n\\begin{th}",
                )],
                main_file: "foo.tex",
                position: Position::new(1, 8),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, Cow::from("theorem"));
        assert_eq!(
            items[0].text_edit.as_ref().map(|edit| edit.range),
            Some(Range::new_simple(1, 7, 1, 9))
        );
    }
}
