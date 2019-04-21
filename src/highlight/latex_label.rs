use crate::feature::FeatureRequest;
use crate::range;
use crate::syntax::latex::analysis::label::{LatexLabel, LatexLabelAnalyzer, LatexLabelKind};
use crate::syntax::latex::ast::LatexVisitor;
use crate::syntax::text::SyntaxNode;
use crate::workspace::SyntaxTree;
use lsp_types::{DocumentHighlight, DocumentHighlightKind, TextDocumentPositionParams};

pub struct LatexLabelHighlightProvider;

impl LatexLabelHighlightProvider {
    pub async fn execute(
        request: &FeatureRequest<TextDocumentPositionParams>,
    ) -> Vec<DocumentHighlight> {
        if let SyntaxTree::Latex(tree) = &request.document.tree {
            let mut analyzer = LatexLabelAnalyzer::new();
            analyzer.visit_root(&tree.root);

            if let Some(name) = analyzer
                .labels
                .iter()
                .find(|label| range::contains(label.name.range(), request.params.position))
                .map(|label| label.name.text())
            {
                return analyzer
                    .labels
                    .iter()
                    .filter(|label| label.name.text() == name)
                    .map(|label| DocumentHighlight {
                        range: label.name.range(),
                        kind: Some(match label.kind {
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
    use crate::feature::FeatureTester;
    use crate::workspace::WorkspaceBuilder;
    use futures::executor;

    #[test]
    fn test_has_label() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "\\label{foo}\n\\ref{foo}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 7, "").into();

        let results = executor::block_on(LatexLabelHighlightProvider::execute(&request));

        let highlight1 = DocumentHighlight {
            range: range::create(0, 7, 0, 10),
            kind: Some(DocumentHighlightKind::Write),
        };
        let highlight2 = DocumentHighlight {
            range: range::create(1, 5, 1, 8),
            kind: Some(DocumentHighlightKind::Read),
        };
        assert_eq!(vec![highlight1, highlight2], results);
    }

    #[test]
    fn test_no_label_latex() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "");
        let request = FeatureTester::new(builder.workspace, uri, 0, 0, "").into();

        let results = executor::block_on(LatexLabelHighlightProvider::execute(&request));

        assert_eq!(results, Vec::new());
    }

    #[test]
    fn test_no_label_bibtex() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.bib", "");
        let request = FeatureTester::new(builder.workspace, uri, 0, 0, "").into();

        let results = executor::block_on(LatexLabelHighlightProvider::execute(&request));

        assert_eq!(results, Vec::new());
    }
}
