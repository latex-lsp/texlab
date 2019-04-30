use crate::feature::FeatureRequest;
use crate::range;
use crate::syntax::latex::analysis::label::{LatexLabelAnalyzer, LatexLabelKind};
use crate::syntax::latex::ast::LatexVisitor;
use crate::syntax::text::SyntaxNode;
use crate::workspace::{Document, SyntaxTree};
use lsp_types::{Location, TextDocumentPositionParams};

pub struct LatexLabelDefinitionProvider;

impl LatexLabelDefinitionProvider {
    pub async fn execute(request: &FeatureRequest<TextDocumentPositionParams>) -> Vec<Location> {
        if let Some(reference) = Self::find_reference(&request) {
            for document in &request.related_documents {
                if let Some(definition) = Self::find_definition(&document, &reference) {
                    return vec![definition];
                }
            }
        }
        Vec::new()
    }

    fn find_definition(document: &Document, reference: &str) -> Option<Location> {
        if let SyntaxTree::Latex(tree) = &document.tree {
            let mut analyzer = LatexLabelAnalyzer::new();
            analyzer.visit_root(&tree.root);
            analyzer
                .labels
                .iter()
                .filter(|label| label.kind == LatexLabelKind::Definition)
                .find(|label| label.name.text() == reference)
                .map(|label| Location::new(document.uri.clone(), label.name.range()))
        } else {
            None
        }
    }

    fn find_reference(request: &FeatureRequest<TextDocumentPositionParams>) -> Option<&str> {
        if let SyntaxTree::Latex(tree) = &request.document.tree {
            let mut analyzer = LatexLabelAnalyzer::new();
            analyzer.visit_root(&tree.root);
            analyzer
                .labels
                .iter()
                .find(|label| range::contains(label.name.range(), request.params.position))
                .map(|label| label.name.text())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;
    use crate::workspace::WorkspaceBuilder;
    use futures::executor::block_on;

    #[test]
    fn test_has_definition() {
        let mut builder = WorkspaceBuilder::new();
        builder.document("foo.tex", "\\label{foo}");
        let uri1 = builder.document("bar.tex", "\\label{foo}\n\\input{baz.tex}");
        let uri2 = builder.document("baz.tex", "\\ref{foo}");
        let request = FeatureTester::new(builder.workspace, uri2, 0, 5, "").into();

        let results = block_on(LatexLabelDefinitionProvider::execute(&request));

        let location = Location::new(uri1, range::create(0, 7, 0, 10));
        assert_eq!(vec![location], results)
    }

    #[test]
    fn test_no_definition_latex() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "");
        let request = FeatureTester::new(builder.workspace, uri, 0, 0, "").into();

        let results = block_on(LatexLabelDefinitionProvider::execute(&request));

        assert_eq!(results, Vec::new());
    }

    #[test]
    fn test_no_definition_bibtex() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.bib", "");
        let request = FeatureTester::new(builder.workspace, uri, 0, 0, "").into();

        let results = block_on(LatexLabelDefinitionProvider::execute(&request));

        assert_eq!(results, Vec::new());
    }
}
