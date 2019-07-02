use crate::completion::factory;
use crate::completion::latex::combinators::{self, Parameter};
use crate::data::language::{language_data, LatexLabelKind};
use crate::feature::{FeatureProvider, FeatureRequest};
use crate::syntax::SyntaxTree;
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams, TextEdit};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexLabelCompletionProvider;

impl FeatureProvider for LatexLabelCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let parameters = language_data()
            .label_commands
            .iter()
            .filter(|cmd| cmd.kind == LatexLabelKind::Reference)
            .map(|cmd| Parameter::new(&cmd.name, cmd.index));

        combinators::argument(request, parameters, async move |context| {
            let mut items = Vec::new();
            for document in request.related_documents() {
                if let SyntaxTree::Latex(tree) = &document.tree {
                    for label in &tree.labels {
                        if label.kind == LatexLabelKind::Definition {
                            for name in label.names() {
                                let text = name.text().to_owned();
                                let text_edit = TextEdit::new(context.range, text.clone().into());
                                let item = factory::label(request, text.into(), text_edit);
                                items.push(item);
                            }
                        }
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
