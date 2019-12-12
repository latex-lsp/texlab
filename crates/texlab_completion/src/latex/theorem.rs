use super::combinators;
use crate::factory::{self, LatexComponentId};
use futures_boxed::boxed;
use texlab_protocol::*;
use texlab_syntax::*;
use texlab_workspace::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexTheoremEnvironmentCompletionProvider;

impl FeatureProvider for LatexTheoremEnvironmentCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        combinators::environment(request, |context| {
            async move {
                let mut items = Vec::new();
                for document in request.related_documents() {
                    if let SyntaxTree::Latex(tree) = &document.tree {
                        for theorem in &tree.math.theorem_definitions {
                            let name = theorem.name().text().to_owned();
                            let text_edit = TextEdit::new(context.range, name.clone());
                            let item = factory::environment(
                                request,
                                name,
                                text_edit,
                                &LatexComponentId::User,
                            );
                            items.push(item);
                        }
                    }
                }
                items
            }
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(items[0].label, "theorem");
        assert_eq!(
            items[0].text_edit.as_ref().map(|edit| edit.range),
            Some(Range::new_simple(1, 7, 1, 9))
        );
    }
}
