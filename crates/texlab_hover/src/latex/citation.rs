use futures_boxed::boxed;
use log::warn;
use texlab_citeproc::render_citation;
use texlab_feature::{DocumentContent, FeatureProvider, FeatureRequest};
use texlab_protocol::{Hover, HoverContents, RangeExt, TextDocumentPositionParams};
use texlab_syntax::{bibtex, Span, SyntaxNode};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexCitationHoverProvider;

impl FeatureProvider for LatexCitationHoverProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Hover>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let (tree, src_key, entry) = Self::get_entry(req)?;
        if entry.is_comment() {
            None
        } else {
            let key = entry.key.as_ref()?;
            match render_citation(&tree, key.text()) {
                Some(markdown) => Some(Hover {
                    contents: HoverContents::Markup(markdown),
                    range: Some(src_key.range()),
                }),
                None => {
                    warn!("Failed to render entry: {}", key.text());
                    None
                }
            }
        }
    }
}

impl LatexCitationHoverProvider {
    fn get_entry(
        req: &FeatureRequest<TextDocumentPositionParams>,
    ) -> Option<(&bibtex::Tree, &Span, &bibtex::Entry)> {
        let key = Self::get_key(req)?;
        for tree in req
            .related()
            .iter()
            .filter_map(|doc| doc.content.as_bibtex())
        {
            for entry in tree
                .children(tree.root)
                .filter_map(|node| tree.as_entry(node))
            {
                if let Some(current_key) = &entry.key {
                    if current_key.text() == key.text {
                        return Some((tree, key, entry));
                    }
                }
            }
        }
        None
    }

    fn get_key(req: &FeatureRequest<TextDocumentPositionParams>) -> Option<&Span> {
        match &req.current().content {
            DocumentContent::Latex(table) => table
                .citations
                .iter()
                .flat_map(|citation| citation.keys(&table))
                .find(|key| key.range().contains(req.params.position))
                .map(|token| &token.span),
            DocumentContent::Bibtex(tree) => tree
                .children(tree.root)
                .filter_map(|node| tree.as_entry(node))
                .filter_map(|entry| entry.key.as_ref())
                .find(|key| key.range().contains(req.params.position))
                .map(|token| &token.span),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use texlab_feature::FeatureTester;
    use texlab_protocol::{MarkupContent, MarkupKind, Range, RangeExt};

    #[tokio::test]
    async fn empty_latex_document() {
        let actual_hover = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_position(LatexCitationHoverProvider)
            .await;

        assert_eq!(actual_hover, None);
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_hover = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_position(LatexCitationHoverProvider)
            .await;

        assert_eq!(actual_hover, None);
    }

    #[tokio::test]
    async fn inside_label() {
        let actual_hover = FeatureTester::new()
            .file(
                "main.bib",
                "@article{foo, author = {Foo Bar}, title = {Baz Qux}, year = 1337}",
            )
            .file(
                "main.tex",
                indoc!(
                    r#"
                        \addbibresource{main.bib}
                        \cite{foo}
                    "#
                ),
            )
            .main("main.tex")
            .position(1, 7)
            .test_position(LatexCitationHoverProvider)
            .await
            .unwrap();

        let expected_hover = Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Bar, F. (1337). *Baz Qux*.".into(),
            }),
            range: Some(Range::new_simple(1, 6, 1, 9)),
        };

        assert_eq!(actual_hover, expected_hover);
    }

    #[tokio::test]
    async fn inside_entry() {
        let actual_hover = FeatureTester::new()
            .file(
                "main.bib",
                "@article{foo, author = {Foo Bar}, title = {Baz Qux}, year = 1337}",
            )
            .file(
                "main.tex",
                indoc!(
                    r#"
                        \addbibresource{main.bib}
                        \cite{foo}
                    "#
                ),
            )
            .main("main.bib")
            .position(0, 11)
            .test_position(LatexCitationHoverProvider)
            .await
            .unwrap();

        let expected_hover = Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Bar, F. (1337). *Baz Qux*.".into(),
            }),
            range: Some(Range::new_simple(0, 9, 0, 12)),
        };

        assert_eq!(actual_hover, expected_hover);
    }
}
