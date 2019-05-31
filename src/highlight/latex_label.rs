use crate::feature::{FeatureProvider, FeatureRequest};
use crate::syntax::latex::*;
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use futures::prelude::*;
use futures_boxed::boxed;
use lsp_types::{DocumentHighlight, DocumentHighlightKind, TextDocumentPositionParams};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexLabelHighlightProvider;

impl FeatureProvider for LatexLabelHighlightProvider {
    type Params = TextDocumentPositionParams;
    type Output = Vec<DocumentHighlight>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<TextDocumentPositionParams>,
    ) -> Vec<DocumentHighlight> {
        if let SyntaxTree::Latex(tree) = &request.document.tree {
            if let Some(name) = tree
                .labels
                .iter()
                .find(|label| label.name().range().contains(request.params.position))
                .map(|label| label.name().text())
            {
                return tree
                    .labels
                    .iter()
                    .filter(|label| label.name().text() == name)
                    .map(|label| DocumentHighlight {
                        range: label.name().range(),
                        kind: Some(match label.kind() {
                            LatexLabelKind::Definition => DocumentHighlightKind::Write,
                            LatexLabelKind::Reference => DocumentHighlightKind::Read,
                        }),
                    })
                    .collect();
            }
        }
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{test_feature, FeatureSpec};
    use lsp_types::{Position, Range};

    #[test]
    fn test_has_label() {
        let highlights = test_feature(
            LatexLabelHighlightProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\label{foo}\n\\ref{foo}")],
                main_file: "foo.tex",
                position: Position::new(0, 7),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            highlights,
            vec![
                DocumentHighlight {
                    range: Range::new_simple(0, 7, 0, 10),
                    kind: Some(DocumentHighlightKind::Write),
                },
                DocumentHighlight {
                    range: Range::new_simple(1, 5, 1, 8),
                    kind: Some(DocumentHighlightKind::Read),
                }
            ]
        );
    }

    #[test]
    fn test_no_label_latex() {
        let highlights = test_feature(
            LatexLabelHighlightProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "")],
                main_file: "foo.tex",
                position: Position::new(0, 0),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(highlights, Vec::new());
    }

    #[test]
    fn test_no_label_bibtex() {
        let highlights = test_feature(
            LatexLabelHighlightProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "")],
                main_file: "foo.bib",
                position: Position::new(0, 0),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(highlights, Vec::new());
    }
}
