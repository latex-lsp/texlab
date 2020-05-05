use crate::{
    feature::{FeatureProvider, FeatureRequest},
    protocol::{
        Hover, HoverContents, MarkupContent, MarkupKind, RangeExt, TextDocumentPositionParams,
    },
    syntax::{SyntaxNode, LANGUAGE_DATA},
};
use async_trait::async_trait;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct BibtexEntryTypeHoverProvider;

#[async_trait]
impl FeatureProvider for BibtexEntryTypeHoverProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Hover>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let tree = req.current().content.as_bibtex()?;
        for entry in tree
            .children(tree.root)
            .filter_map(|node| tree.as_entry(node))
        {
            if entry.ty.range().contains(req.params.position) {
                let ty = &entry.ty.text()[1..];
                let docs = LANGUAGE_DATA.entry_type_documentation(ty)?;
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: docs.into(),
                    }),
                    range: Some(entry.ty.range()),
                });
            }
        }
        None
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
            .test_position(BibtexEntryTypeHoverProvider)
            .await;

        assert_eq!(actual_hover, None);
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_hover = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_position(BibtexEntryTypeHoverProvider)
            .await;

        assert_eq!(actual_hover, None);
    }

    #[tokio::test]
    async fn known_entry_type() {
        let actual_hover = FeatureTester::new()
            .file("main.bib", "@article{foo,}")
            .main("main.bib")
            .position(0, 3)
            .test_position(BibtexEntryTypeHoverProvider)
            .await
            .unwrap();

        let expected_hover = Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: LANGUAGE_DATA
                    .entry_type_documentation("article")
                    .unwrap()
                    .into(),
            }),
            range: Some(Range::new_simple(0, 0, 0, 8)),
        };

        assert_eq!(actual_hover, expected_hover);
    }

    #[tokio::test]
    async fn unknown_entry_type() {
        let actual_hover = FeatureTester::new()
            .file("main.bib", "@foo{bar,}")
            .main("main.bib")
            .position(0, 3)
            .test_position(BibtexEntryTypeHoverProvider)
            .await;

        assert_eq!(actual_hover, None);
    }

    #[tokio::test]
    async fn entry_key() {
        let actual_hover = FeatureTester::new()
            .file("main.bib", "@article{foo,}")
            .main("main.bib")
            .position(0, 11)
            .test_position(BibtexEntryTypeHoverProvider)
            .await;

        assert_eq!(actual_hover, None);
    }
}
