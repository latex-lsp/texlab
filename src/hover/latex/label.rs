use crate::{
    feature::{DocumentView, FeatureProvider, FeatureRequest},
    outline::{Outline, OutlineContext},
    workspace::{Document, DocumentContent},
};
use futures_boxed::boxed;
use std::sync::Arc;
use texlab_protocol::{Hover, HoverContents, Position, RangeExt, TextDocumentPositionParams};
use texlab_syntax::{latex, LatexLabelKind, SyntaxNode};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexLabelHoverProvider;

impl FeatureProvider for LatexLabelHoverProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Hover>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let table = req.current().content.as_latex()?;
        let reference = Self::find_reference(table, req.params.position)?;
        let (doc, def) = Self::find_definition(&req.view, reference)?;

        let snapshot = Arc::clone(&req.view.snapshot);
        let view = DocumentView::analyze(snapshot, doc, &req.options, &req.current_dir);
        let outline = Outline::analyze(&view, &req.options, &req.current_dir);
        let outline_ctx = OutlineContext::parse(&view, &outline, def)?;
        let markup = outline_ctx.documentation();
        Some(Hover {
            contents: HoverContents::Markup(markup),
            range: Some(reference.range()),
        })
    }
}

impl LatexLabelHoverProvider {
    fn find_reference(table: &latex::SymbolTable, pos: Position) -> Option<&latex::Token> {
        for label in &table.labels {
            let names = label.names(&table.tree);
            if names.len() == 1 && table.tree.range(label.parent).contains(pos) {
                return Some(&label.names(&table.tree)[0]);
            }

            for name in &names {
                if name.range().contains(pos) {
                    return Some(name);
                }
            }
        }
        None
    }

    fn find_definition(
        view: &DocumentView,
        reference: &latex::Token,
    ) -> Option<(Arc<Document>, latex::Label)> {
        for doc in &view.related {
            if let DocumentContent::Latex(table) = &doc.content {
                for label in &table.labels {
                    if label.kind == LatexLabelKind::Definition {
                        for name in label.names(&table.tree) {
                            if name.text() == reference.text() {
                                return Some((Arc::clone(&doc), *label));
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;
    use texlab_protocol::{Range, RangeExt};

    #[tokio::test]
    async fn empty_latex_document() {
        let actual_hover = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_position(LatexLabelHoverProvider)
            .await;

        assert_eq!(actual_hover, None);
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_hover = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_position(LatexLabelHoverProvider)
            .await;

        assert_eq!(actual_hover, None);
    }

    #[tokio::test]
    async fn section() {
        let actual_hover = FeatureTester::new()
            .file("main.tex", r#"\section{Foo}\label{sec:foo}"#)
            .main("main.tex")
            .position(0, 23)
            .test_position(LatexLabelHoverProvider)
            .await
            .unwrap();

        assert_eq!(actual_hover.range.unwrap(), Range::new_simple(0, 20, 0, 27));
    }
}
