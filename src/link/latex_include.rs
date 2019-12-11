use crate::workspace::*;
use futures_boxed::boxed;
use texlab_protocol::{DocumentLink, DocumentLinkParams};
use texlab_syntax::*;

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
        if let SyntaxTree::Latex(tree) = &request.document().tree {
            return tree
                .includes
                .iter()
                .flat_map(|include| Self::resolve(request, include))
                .collect();
        }
        Vec::new()
    }
}

impl LatexIncludeLinkProvider {
    fn resolve(
        request: &FeatureRequest<DocumentLinkParams>,
        include: &LatexInclude,
    ) -> Vec<DocumentLink> {
        let mut links = Vec::new();
        let paths = include.paths();
        for (i, targets) in include.all_targets.iter().enumerate() {
            for target in targets {
                if let Some(link) = request
                    .workspace()
                    .find(target)
                    .map(|document| DocumentLink {
                        range: paths[i].range(),
                        target: document.uri.clone().into(),
                    })
                {
                    links.push(link);
                }
            }
        }
        links
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use texlab_protocol::RangeExt;
    use texlab_protocol::{Position, Range};

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
        assert!(links.is_empty());
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
        assert!(links.is_empty());
    }
}
