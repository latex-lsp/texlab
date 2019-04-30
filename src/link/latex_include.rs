use crate::feature::FeatureRequest;
use crate::syntax::latex::analysis::include::{LatexInclude, LatexIncludeAnalyzer};
use crate::syntax::latex::ast::LatexVisitor;
use crate::syntax::text::SyntaxNode;
use crate::workspace::SyntaxTree;
use lsp_types::{DocumentLink, DocumentLinkParams};

pub struct LatexIncludeLinkProvider;

impl LatexIncludeLinkProvider {
    pub async fn execute(request: &FeatureRequest<DocumentLinkParams>) -> Vec<DocumentLink> {
        if let SyntaxTree::Latex(tree) = &request.document.tree {
            let mut analyzer = LatexIncludeAnalyzer::new();
            analyzer.visit_root(&tree.root);
            return analyzer
                .included_files
                .iter()
                .flat_map(|include| Self::resolve(&request, &include))
                .collect();
        }
        Vec::new()
    }

    fn resolve(
        request: &FeatureRequest<DocumentLinkParams>,
        include: &LatexInclude,
    ) -> Option<DocumentLink> {
        request
            .workspace
            .resolve_document(&request.document.uri, include.path.text())
            .map(|target| DocumentLink {
                range: include.path.range(),
                target: target.uri.clone(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;
    use crate::range;
    use crate::workspace::WorkspaceBuilder;
    use futures::executor::block_on;

    #[test]
    fn test_has_links() {
        let mut builder = WorkspaceBuilder::new();
        let uri1 = builder.document("foo.tex", "\\input{bar.tex}");
        let uri2 = builder.document("bar.tex", "");
        let request = FeatureTester::new(builder.workspace, uri1, 0, 0, "").into();

        let results = block_on(LatexIncludeLinkProvider::execute(&request));

        let link = DocumentLink {
            range: range::create(0, 7, 0, 14),
            target: uri2,
        };
        assert_eq!(vec![link], results)
    }

    #[test]
    fn test_no_links_latex() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "");
        let request = FeatureTester::new(builder.workspace, uri, 0, 0, "").into();

        let results = block_on(LatexIncludeLinkProvider::execute(&request));

        assert_eq!(results, Vec::new());
    }

    #[test]
    fn test_no_links_bibtex() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.bib", "");
        let request = FeatureTester::new(builder.workspace, uri, 0, 0, "").into();

        let results = block_on(LatexIncludeLinkProvider::execute(&request));

        assert_eq!(results, Vec::new());
    }
}
