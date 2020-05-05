use crate::{
    feature::{FeatureProvider, FeatureRequest},
    protocol::{DocumentLink, DocumentLinkParams},
    syntax::{latex, SyntaxNode},
    workspace::DocumentContent,
};
use async_trait::async_trait;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexImportLinkProvider;

#[async_trait]
impl FeatureProvider for LatexImportLinkProvider {
    type Params = DocumentLinkParams;
    type Output = Vec<DocumentLink>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        if let DocumentContent::Latex(table) = &req.current().content {
            table
                .imports
                .iter()
                .flat_map(|import| Self::resolve(req, table, import))
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl LatexImportLinkProvider {
    fn resolve(
        req: &FeatureRequest<DocumentLinkParams>,
        table: &latex::SymbolTable,
        import: &latex::Import,
    ) -> Vec<DocumentLink> {
        let mut links = Vec::new();
        let file = import.file(&table);
        for target in &import.targets {
            if let Some(link) = req.snapshot().find(target).map(|doc| DocumentLink {
                range: file.range(),
                target: doc.uri.clone().into(),
                tooltip: None,
            }) {
                links.push(link);
                break;
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
            .test_link(LatexImportLinkProvider)
            .await;

        assert!(actual_links.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document_command() {
        let actual_links = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .test_link(LatexImportLinkProvider)
            .await;

        assert!(actual_links.is_empty());
    }

    #[tokio::test]
    async fn has_links() {
        let actual_links = FeatureTester::new()
            .file("foo.tex", r#"\import{bar/}{baz}"#)
            .file("bar/baz.tex", r#""#)
            .main("foo.tex")
            .test_link(LatexImportLinkProvider)
            .await;

        let expected_links = vec![DocumentLink {
            range: Range::new_simple(0, 14, 0, 17),
            target: FeatureTester::uri("bar/baz.tex").into(),
            tooltip: None,
        }];

        assert_eq!(actual_links, expected_links);
    }
}
