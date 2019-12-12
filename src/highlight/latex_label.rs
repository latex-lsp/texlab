use futures_boxed::boxed;
use texlab_protocol::RangeExt;
use texlab_protocol::{DocumentHighlight, DocumentHighlightKind, TextDocumentPositionParams};
use texlab_syntax::*;
use texlab_workspace::*;

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
        let mut highlights = Vec::new();
        if let SyntaxTree::Latex(tree) = &request.document().tree {
            if let Some(name) = tree
                .structure
                .labels
                .iter()
                .flat_map(LatexLabel::names)
                .find(|label| label.range().contains(request.params.position))
                .map(|label| label.text())
            {
                for label_group in &tree.structure.labels {
                    for label in label_group.names() {
                        if label.text() == name {
                            let highlight = DocumentHighlight {
                                range: label.range(),
                                kind: Some(match label_group.kind {
                                    LatexLabelKind::Definition => DocumentHighlightKind::Write,
                                    LatexLabelKind::Reference(_) => DocumentHighlightKind::Read,
                                }),
                            };
                            highlights.push(highlight);
                        }
                    }
                }
            }
        }
        highlights
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use texlab_protocol::{Position, Range};

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
        assert!(highlights.is_empty());
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
        assert!(highlights.is_empty());
    }
}
