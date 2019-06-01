use crate::feature::{FeatureProvider, FeatureRequest};
use crate::syntax::latex::*;
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use futures_boxed::boxed;
use lsp_types::{DocumentLink, DocumentLinkParams};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexIncludeLinkProvider;

impl FeatureProvider for LatexIncludeLinkProvider {
    type Params = DocumentLinkParams;
    type Output = Vec<DocumentLink>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<DocumentLinkParams>,
    ) -> Vec<DocumentLink> {
        if let SyntaxTree::Latex(tree) = &request.document.tree {
            return tree
                .includes
                .iter()
                .flat_map(|include| Self::resolve(&request, &include))
                .collect();
        }
        Vec::new()
    }
}

impl LatexIncludeLinkProvider {
    fn resolve(
        request: &FeatureRequest<DocumentLinkParams>,
        include: &LatexInclude,
    ) -> Option<DocumentLink> {
        request
            .workspace
            .find(include.target())
            .map(|target| DocumentLink {
                range: include.path().range(),
                target: target.uri.clone(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{test_feature, FeatureSpec};
    use lsp_types::{Position, Range};

    #[test]
    fn test_has_links() {
        let links = test_feature(
            LatexIncludeLinkProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\input{bar.tex}"),
                    FeatureSpec::file("bar.tex", ""),
                ],
                main_file: "foo.tex",
                position: Position::new(0, 15),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            links,
            vec![DocumentLink {
                range: Range::new_simple(0, 7, 0, 14),
                target: FeatureSpec::uri("bar.tex"),
            }]
        );
    }

    #[test]
    fn test_no_links_latex() {
        let links = test_feature(
            LatexIncludeLinkProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "")],
                main_file: "foo.tex",
                position: Position::new(0, 15),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(links, Vec::new());
    }

    #[test]
    fn test_no_links_bibtex() {
        let links = test_feature(
            LatexIncludeLinkProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "")],
                main_file: "foo.bib",
                position: Position::new(0, 15),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(links, Vec::new());
    }
}
