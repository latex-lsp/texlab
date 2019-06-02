use crate::completion::factory;
use crate::completion::latex::combinators::LatexCombinators;
use crate::feature::{FeatureProvider, FeatureRequest};
use crate::syntax::latex::*;
use crate::syntax::SyntaxTree;
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};
use std::borrow::Cow;

pub struct LatexLabelCompletionProvider;

impl FeatureProvider for LatexLabelCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<CompletionParams>,
    ) -> Vec<CompletionItem> {
        LatexCombinators::argument(request, &LABEL_REFERENCE_COMMANDS, 0, async move |_| {
            let mut items = Vec::new();
            for document in &request.related_documents {
                if let SyntaxTree::Latex(tree) = &document.tree {
                    tree.labels
                        .iter()
                        .filter(|label| label.kind() == LatexLabelKind::Definition)
                        .map(|label| Cow::from(label.name().text().to_owned()))
                        .map(factory::create_label)
                        .for_each(|item| items.push(item))
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
    fn test_inside_of_ref() {
        let items = test_feature(
            LatexLabelCompletionProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file(
                        "foo.tex",
                        "\\addbibresource{bar.bib}\\include{baz}\n\\ref{}",
                    ),
                    FeatureSpec::file("bar.bib", ""),
                    FeatureSpec::file("baz.tex", "\\label{foo}\\label{bar}\\ref{baz}"),
                ],
                main_file: "foo.tex",
                position: Position::new(1, 5),
                ..FeatureSpec::default()
            },
        );
        let labels: Vec<&str> = items.iter().map(|item| item.label.as_ref()).collect();
        assert_eq!(labels, vec!["foo", "bar"]);
    }

    #[test]
    fn test_outside_of_ref() {
        let items = test_feature(
            LatexLabelCompletionProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\include{bar}\\ref{}"),
                    FeatureSpec::file("bar.tex", "\\label{foo}\\label{bar}"),
                ],
                main_file: "foo.tex",
                position: Position::new(1, 6),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }
}
