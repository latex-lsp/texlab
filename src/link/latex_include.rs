use crate::{
    feature::{FeatureProvider, FeatureRequest},
    protocol::{DocumentLink, DocumentLinkParams},
    syntax::{latex, SyntaxNode},
    workspace::DocumentContent,
};
use async_trait::async_trait;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexIncludeLinkProvider;

#[async_trait]
impl FeatureProvider for LatexIncludeLinkProvider {
    type Params = DocumentLinkParams;
    type Output = Vec<DocumentLink>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        if let DocumentContent::Latex(table) = &req.current().content {
            table
                .includes
                .iter()
                .flat_map(|include| Self::resolve(req, table, include))
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl LatexIncludeLinkProvider {
    fn resolve(
        req: &FeatureRequest<DocumentLinkParams>,
        table: &latex::SymbolTable,
        include: &latex::Include,
    ) -> Vec<DocumentLink> {
        let mut links = Vec::new();
        let paths = include.paths(&table);
        for (i, targets) in include.all_targets.iter().enumerate() {
            for target in targets {
                if let Some(link) = req.snapshot().find(target).map(|doc| DocumentLink {
                    range: paths[i].range(),
                    target: doc.uri.clone().into(),
                    tooltip: None,
                }) {
                    links.push(link);
                    break;
                }
            }
        }
        links
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        feature::FeatureTester,
        protocol::{Range, RangeExt},
    };

    #[tokio::test]
    async fn empty_latex_document_command() {
        let actual_links = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .test_link(LatexIncludeLinkProvider)
            .await;

        assert!(actual_links.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document_command() {
        let actual_links = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .test_link(LatexIncludeLinkProvider)
            .await;

        assert!(actual_links.is_empty());
    }

    #[tokio::test]
    async fn has_links() {
        let actual_links = FeatureTester::new()
            .file("foo.tex", r#"\input{bar.tex}"#)
            .file("bar.tex", r#""#)
            .main("foo.tex")
            .test_link(LatexIncludeLinkProvider)
            .await;

        let expected_links = vec![DocumentLink {
            range: Range::new_simple(0, 7, 0, 14),
            target: FeatureTester::uri("bar.tex").into(),
            tooltip: None,
        }];

        assert_eq!(actual_links, expected_links);
    }
}
