use crate::{
    feature::{FeatureProvider, FeatureRequest},
    protocol::{
        Hover, HoverContents, MarkupContent, MarkupKind, RangeExt, TextDocumentPositionParams,
    },
    syntax::{SyntaxNode, LANGUAGE_DATA},
};
use async_trait::async_trait;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct BibtexFieldHoverProvider;

#[async_trait]
impl FeatureProvider for BibtexFieldHoverProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Hover>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let tree = req.current().content.as_bibtex()?;
        let name = tree
            .find(req.params.position)
            .into_iter()
            .filter_map(|node| tree.as_field(node))
            .map(|field| &field.name)
            .find(|name| name.range().contains(req.params.position))?;

        let docs = LANGUAGE_DATA.field_documentation(name.text())?;
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: docs.into(),
            }),
            range: Some(name.range()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{feature::FeatureTester, protocol::Range};

    #[tokio::test]
    async fn empty_latex_document() {
        let actual_hover = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_position(BibtexFieldHoverProvider)
            .await;

        assert_eq!(actual_hover, None);
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_hover = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_position(BibtexFieldHoverProvider)
            .await;

        assert_eq!(actual_hover, None);
    }

    #[tokio::test]
    async fn known_field() {
        let actual_hover = FeatureTester::new()
            .file("main.bib", "@article{foo, author = bar}")
            .main("main.bib")
            .position(0, 15)
            .test_position(BibtexFieldHoverProvider)
            .await
            .unwrap();

        let expected_hover = Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: LANGUAGE_DATA.field_documentation("author").unwrap().into(),
            }),
            range: Some(Range::new_simple(0, 14, 0, 20)),
        };

        assert_eq!(actual_hover, expected_hover);
    }

    #[tokio::test]
    async fn unknown_field() {
        let actual_hover = FeatureTester::new()
            .file("main.bib", "@article{foo, bar = baz}")
            .main("main.bib")
            .position(0, 15)
            .test_position(BibtexFieldHoverProvider)
            .await;

        assert_eq!(actual_hover, None);
    }

    #[tokio::test]
    async fn entry_key() {
        let actual_hover = FeatureTester::new()
            .file("main.bib", "@article{foo, bar = baz}")
            .main("main.bib")
            .position(0, 11)
            .test_position(BibtexFieldHoverProvider)
            .await;

        assert_eq!(actual_hover, None);
    }
}
