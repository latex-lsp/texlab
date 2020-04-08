use crate::{
    components::COMPONENT_DATABASE,
    feature::{FeatureProvider, FeatureRequest},
    protocol::{Hover, HoverContents, RangeExt, TextDocumentPositionParams},
    syntax::{LatexIncludeKind, SyntaxNode},
};
use futures_boxed::boxed;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexComponentHoverProvider;

impl FeatureProvider for LatexComponentHoverProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Hover>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let table = req.current().content.as_latex()?;
        for include in &table.includes {
            match include.kind {
                LatexIncludeKind::Package | LatexIncludeKind::Class => {
                    for path in include.paths(&table.tree) {
                        if path.range().contains(req.params.position) {
                            let docs = COMPONENT_DATABASE.documentation(path.text())?;
                            return Some(Hover {
                                contents: HoverContents::Markup(docs),
                                range: Some(path.range()),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        None
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
    async fn empty_latex_document() {
        let actual_hover = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_position(LatexComponentHoverProvider)
            .await;

        assert_eq!(actual_hover, None);
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_hover = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_position(LatexComponentHoverProvider)
            .await;

        assert_eq!(actual_hover, None);
    }

    #[tokio::test]
    async fn known_package() {
        let actual_hover = FeatureTester::new()
            .file("main.tex", r#"\usepackage{amsmath}"#)
            .main("main.tex")
            .position(0, 15)
            .test_position(LatexComponentHoverProvider)
            .await
            .unwrap();

        assert_eq!(actual_hover.range.unwrap(), Range::new_simple(0, 12, 0, 19));
    }

    #[tokio::test]
    async fn unknown_class() {
        let actual_hover = FeatureTester::new()
            .file("main.tex", r#"\documentclass{abcdefghijklmnop}"#)
            .main("main.tex")
            .position(0, 20)
            .test_position(LatexComponentHoverProvider)
            .await;

        assert_eq!(actual_hover, None);
    }
}
