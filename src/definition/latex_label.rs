use crate::feature::{FeatureProvider, FeatureRequest};
use crate::syntax::latex::*;
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use crate::workspace::Document;
use futures_boxed::boxed;
use lsp_types::{Location, TextDocumentPositionParams};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexLabelDefinitionProvider;

impl FeatureProvider for LatexLabelDefinitionProvider {
    type Params = TextDocumentPositionParams;
    type Output = Vec<Location>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<TextDocumentPositionParams>,
    ) -> Vec<Location> {
        if let Some(reference) = Self::find_reference(&request) {
            for document in request.related_documents() {
                if let Some(definition) = Self::find_definition(&document, &reference) {
                    return vec![definition];
                }
            }
        }
        Vec::new()
    }
}

impl LatexLabelDefinitionProvider {
    fn find_definition(document: &Document, reference: &str) -> Option<Location> {
        if let SyntaxTree::Latex(tree) = &document.tree {
            tree.labels
                .iter()
                .filter(|label| label.kind() == LatexLabelKind::Definition)
                .find(|label| label.name().text() == reference)
                .map(|label| Location::new(document.uri.clone(), label.name().range()))
        } else {
            None
        }
    }

    fn find_reference(request: &FeatureRequest<TextDocumentPositionParams>) -> Option<&str> {
        if let SyntaxTree::Latex(tree) = &request.document().tree {
            tree.labels
                .iter()
                .find(|label| label.name().range().contains(request.params.position))
                .map(|label| label.name().text())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{test_feature, FeatureSpec};
    use lsp_types::{Position, Range};

    #[test]
    fn test_has_definition() {
        let locations = test_feature(
            LatexLabelDefinitionProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\label{foo}"),
                    FeatureSpec::file("bar.tex", "\\label{foo}\n\\input{baz.tex}"),
                    FeatureSpec::file("baz.tex", "\\ref{foo}"),
                ],
                main_file: "baz.tex",
                position: Position::new(0, 5),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            locations,
            vec![Location::new(
                FeatureSpec::uri("bar.tex"),
                Range::new_simple(0, 7, 0, 10)
            )]
        );
    }

    #[test]
    fn test_no_definition_latex() {
        let locations = test_feature(
            LatexLabelDefinitionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "")],
                main_file: "foo.tex",
                position: Position::new(0, 0),
                ..FeatureSpec::default()
            },
        );
        assert!(locations.is_empty());
    }

    #[test]
    fn test_no_definition_bibtex() {
        let locations = test_feature(
            LatexLabelDefinitionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "")],
                main_file: "foo.bib",
                position: Position::new(0, 0),
                ..FeatureSpec::default()
            },
        );
        assert!(locations.is_empty());
    }
}
