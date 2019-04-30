use crate::feature::FeatureRequest;
use crate::range;
use crate::syntax::latex::analysis::label::{LatexLabelAnalyzer, LatexLabelKind};
use crate::syntax::latex::ast::LatexVisitor;
use crate::workspace::SyntaxTree;
use lsp_types::{Location, ReferenceParams};

pub struct LatexLabelReferenceProvider;

impl LatexLabelReferenceProvider {
    pub async fn execute(request: &FeatureRequest<ReferenceParams>) -> Vec<Location> {
        let mut references = Vec::new();
        if let Some(definition) = Self::find_definition(request) {
            for document in &request.related_documents {
                if let SyntaxTree::Latex(tree) = &document.tree {
                    let mut analyzer = LatexLabelAnalyzer::new();
                    analyzer.visit_root(&tree.root);
                    analyzer
                        .labels
                        .iter()
                        .filter(|label| label.kind == LatexLabelKind::Reference)
                        .filter(|label| label.name.text() == definition)
                        .map(|label| Location::new(document.uri.clone(), label.command.range))
                        .for_each(|location| references.push(location))
                }
            }
        }
        references
    }

    fn find_definition(request: &FeatureRequest<ReferenceParams>) -> Option<&str> {
        if let SyntaxTree::Latex(tree) = &request.document.tree {
            let mut analyzer = LatexLabelAnalyzer::new();
            analyzer.visit_root(&tree.root);
            analyzer
                .labels
                .iter()
                .find(|label| {
                    label.kind == LatexLabelKind::Definition
                        && range::contains(label.command.range, request.params.position)
                })
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
    fn test() {
        let mut builder = WorkspaceBuilder::new();
        let uri1 = builder.document("foo.tex", "\\label{foo}");
        let uri2 = builder.document("bar.tex", "\\input{foo.tex}\n\\ref{foo}");
        builder.document("baz.tex", "\\ref{foo}");
        let request = FeatureTester::new(builder.workspace, uri1, 0, 8, "").into();

        let results = block_on(LatexLabelReferenceProvider::execute(&request));

        let location = Location::new(uri2, range::create(1, 0, 1, 9));
        assert_eq!(vec![location], results);
    }

    #[test]
    fn test_bibtex() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.bib", "");
        let request = FeatureTester::new(builder.workspace, uri, 0, 0, "").into();

        let results = block_on(LatexLabelReferenceProvider::execute(&request));

        assert_eq!(results, Vec::new());
    }
}
