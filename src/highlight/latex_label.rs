use futures_boxed::boxed;
use texlab_feature::{DocumentContent, FeatureProvider, FeatureRequest};
use texlab_protocol::{
    DocumentHighlight, DocumentHighlightKind, RangeExt, TextDocumentPositionParams,
};
use texlab_syntax::{latex, LatexLabelKind, SyntaxNode};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexLabelHighlightProvider;

impl FeatureProvider for LatexLabelHighlightProvider {
    type Params = TextDocumentPositionParams;
    type Output = Vec<DocumentHighlight>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut highlights = Vec::new();
        if let DocumentContent::Latex(table) = &req.current().content {
            if let Some(name) = table
                .labels
                .iter()
                .flat_map(|label| label.names(&table.tree))
                .find(|label| label.range().contains(req.params.position))
                .map(latex::Token::text)
            {
                for label_group in &table.labels {
                    for label in label_group.names(&table.tree) {
                        if label.text() == name {
                            let kind = match label_group.kind {
                                LatexLabelKind::Definition => DocumentHighlightKind::Write,
                                LatexLabelKind::Reference(_) => DocumentHighlightKind::Read,
                            };

                            let highlight = DocumentHighlight {
                                range: label.range(),
                                kind: Some(kind),
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
    use indoc::indoc;
    use texlab_feature::FeatureTester;
    use texlab_protocol::Range;

    #[tokio::test]
    async fn has_label() {
        let actual_highlights = FeatureTester::new()
            .file(
                "foo.tex",
                indoc!(
                    r#"
                        \label{foo}
                        \ref{foo}
                    "#
                ),
            )
            .main("foo.tex")
            .position(0, 7)
            .test_position(LatexLabelHighlightProvider)
            .await;

        let expected_highlights = vec![
            DocumentHighlight {
                range: Range::new_simple(0, 7, 0, 10),
                kind: Some(DocumentHighlightKind::Write),
            },
            DocumentHighlight {
                range: Range::new_simple(1, 5, 1, 8),
                kind: Some(DocumentHighlightKind::Read),
            },
        ];

        assert_eq!(actual_highlights, expected_highlights);
    }

    #[tokio::test]
    async fn no_label_latex() {
        let actual_highlights = FeatureTester::new()
            .file("foo.tex", "")
            .main("foo.tex")
            .position(0, 0)
            .test_position(LatexLabelHighlightProvider)
            .await;

        assert!(actual_highlights.is_empty());
    }

    #[tokio::test]
    async fn no_label_bibtex() {
        let actual_highlights = FeatureTester::new()
            .file("foo.bib", "")
            .main("foo.bib")
            .position(0, 0)
            .test_position(LatexLabelHighlightProvider)
            .await;

        assert!(actual_highlights.is_empty());
    }
}
