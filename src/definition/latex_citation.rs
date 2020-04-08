use crate::{
    feature::{FeatureProvider, FeatureRequest},
    protocol::{LocationLink, RangeExt, TextDocumentPositionParams},
    syntax::{latex, SyntaxNode},
    workspace::{Document, DocumentContent},
};
use futures_boxed::boxed;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexCitationDefinitionProvider;

impl FeatureProvider for LatexCitationDefinitionProvider {
    type Params = TextDocumentPositionParams;
    type Output = Vec<LocationLink>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut links = Vec::new();
        if let Some(reference) = Self::find_reference(req) {
            for doc in req.related() {
                Self::find_definitions(&doc, &reference, &mut links);
            }
        }
        links
    }
}

impl LatexCitationDefinitionProvider {
    fn find_reference(req: &FeatureRequest<TextDocumentPositionParams>) -> Option<&latex::Token> {
        req.current().content.as_latex().and_then(|table| {
            table
                .citations
                .iter()
                .flat_map(|citation| citation.keys(&table.tree))
                .find(|key| key.range().contains(req.params.position))
        })
    }

    fn find_definitions(doc: &Document, reference: &latex::Token, links: &mut Vec<LocationLink>) {
        if let DocumentContent::Bibtex(tree) = &doc.content {
            for entry in tree
                .children(tree.root)
                .filter_map(|node| tree.as_entry(node))
            {
                if let Some(key) = &entry.key {
                    if key.text() == reference.text() {
                        links.push(LocationLink {
                            origin_selection_range: Some(reference.range()),
                            target_uri: doc.uri.clone().into(),
                            target_range: entry.range(),
                            target_selection_range: key.range(),
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{feature::FeatureTester, protocol::Range};
    use indoc::indoc;

    #[tokio::test]
    async fn empty_latex_document() {
        let actual_links = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_position(LatexCitationDefinitionProvider)
            .await;

        assert!(actual_links.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_links = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_position(LatexCitationDefinitionProvider)
            .await;

        assert!(actual_links.is_empty());
    }

    #[tokio::test]
    async fn has_definition() {
        let actual_links = FeatureTester::new()
            .file(
                "foo.tex",
                indoc!(
                    r#"
                        \addbibresource{baz.bib}
                        \cite{foo}
                    "#
                ),
            )
            .file("bar.bib", r#"@article{foo, bar = {baz}}"#)
            .file("baz.bib", r#"@article{foo, bar = {baz}}"#)
            .main("foo.tex")
            .position(1, 6)
            .test_position(LatexCitationDefinitionProvider)
            .await;

        let exepcted_links = vec![LocationLink {
            origin_selection_range: Some(Range::new_simple(1, 6, 1, 9)),
            target_uri: FeatureTester::uri("baz.bib").into(),
            target_range: Range::new_simple(0, 0, 0, 26),
            target_selection_range: Range::new_simple(0, 9, 0, 12),
        }];

        assert_eq!(actual_links, exepcted_links);
    }
}
