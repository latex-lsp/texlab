use super::combinators::{self, ArgumentContext, Parameter};
use crate::completion::factory;
use crate::syntax::*;
use crate::workspace::*;
use futures_boxed::boxed;
use lsp_types::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexLabelCompletionProvider;

impl FeatureProvider for LatexLabelCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let parameters = LANGUAGE_DATA
            .label_commands
            .iter()
            .filter(|cmd| cmd.kind.is_reference())
            .map(|cmd| Parameter::new(&cmd.name, cmd.index));

        combinators::argument(request, parameters, async move |context| {
            let outline = Outline::from(&request.view);
            let source = Self::find_source(&context);
            let mut items = Vec::new();
            for document in request.related_documents() {
                if let SyntaxTree::Latex(tree) = &document.tree {
                    for label in tree
                        .labels
                        .iter()
                        .filter(|label| label.kind == LatexLabelKind::Definition)
                        .filter(|label| Self::is_included(tree, label, source))
                    {
                        let outline_ctx =
                            OutlineContext::find(&outline, &request.view, label.start());
                        for name in label.names() {
                            let text = name.text().to_owned();
                            let text_edit = TextEdit::new(context.range, text.clone().into());
                            let item =
                                factory::label(request, text.into(), text_edit, &outline_ctx);
                            items.push(item);
                        }
                    }
                }
            }
            items
        })
        .await
    }
}

impl LatexLabelCompletionProvider {
    fn find_source(context: &ArgumentContext) -> LatexLabelReferenceSource {
        match LANGUAGE_DATA
            .label_commands
            .iter()
            .find(|cmd| cmd.name == context.parameter.name && cmd.index == context.parameter.index)
            .map(|cmd| cmd.kind)
            .unwrap()
        {
            LatexLabelKind::Definition => unreachable!(),
            LatexLabelKind::Reference(source) => source,
        }
    }

    fn is_included(
        tree: &LatexSyntaxTree,
        label: &LatexLabel,
        source: LatexLabelReferenceSource,
    ) -> bool {
        match source {
            LatexLabelReferenceSource::Everything => true,
            LatexLabelReferenceSource::Math => tree
                .environments
                .iter()
                .filter(|env| env.left.is_math())
                .any(|env| env.range().contains_exclusive(label.start())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn test_eqref() {
        let items = test_feature(
            LatexLabelCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.tex",
                    "\\begin{align}\\label{foo}\\end{align}\\label{bar}\n\\eqref{}",
                )],
                main_file: "foo.tex",
                position: Position::new(1, 7),
                ..FeatureSpec::default()
            },
        );
        let labels: Vec<&str> = items.iter().map(|item| item.label.as_ref()).collect();
        assert_eq!(labels, vec!["foo"]);
    }
}
