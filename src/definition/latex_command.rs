use crate::feature::{FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use lsp_types::{Location, TextDocumentPositionParams};
use texlab_syntax::*;

pub struct LatexCommandDefinitionProvider;

impl FeatureProvider for LatexCommandDefinitionProvider {
    type Params = TextDocumentPositionParams;
    type Output = Vec<Location>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut definitions = Vec::new();
        if let SyntaxTree::Latex(tree) = &request.document().tree {
            if let Some(LatexNode::Command(command)) = tree.find(request.params.position).last() {
                for document in request.related_documents() {
                    if let SyntaxTree::Latex(tree) = &document.tree {
                        tree.command_definitions
                            .iter()
                            .filter(|def| def.definition.name.text() == command.name.text())
                            .map(|def| Location::new(document.uri.clone(), def.range()))
                            .for_each(|def| definitions.push(def));

                        tree.math_operators
                            .iter()
                            .filter(|op| op.definition.name.text() == command.name.text())
                            .map(|op| Location::new(document.uri.clone(), op.range()))
                            .for_each(|def| definitions.push(def));
                    }
                }
            }
        }
        definitions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{test_feature, FeatureSpec};
    use lsp_types::{Position, Range};

    #[test]
    fn test_command_definition() {
        let locations = test_feature(
            LatexCommandDefinitionProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\include{bar}\n\\foo"),
                    FeatureSpec::file("bar.tex", "\\newcommand{\\foo}{bar}"),
                    FeatureSpec::file("baz.tex", "\\newcommand{\\foo}{baz}"),
                ],
                main_file: "foo.tex",
                position: Position::new(1, 3),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            locations,
            vec![Location::new(
                FeatureSpec::uri("bar.tex"),
                Range::new_simple(0, 0, 0, 22)
            )]
        );
    }

    #[test]
    fn test_math_operator() {
        let locations = test_feature(
            LatexCommandDefinitionProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\include{bar}\n\\foo"),
                    FeatureSpec::file("bar.tex", "\\DeclareMathOperator{\\foo}{bar}"),
                    FeatureSpec::file("baz.tex", "\\DeclareMathOperator{\\foo}{baz}"),
                ],
                main_file: "foo.tex",
                position: Position::new(1, 3),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            locations,
            vec![Location::new(
                FeatureSpec::uri("bar.tex"),
                Range::new_simple(0, 0, 0, 31)
            )]
        );
    }
}
