use crate::{
    feature::{DocumentView, FeatureProvider, FeatureRequest},
    outline::{Outline, OutlineContext, OutlineContextItem},
    protocol::{LocationLink, Options, RangeExt, TextDocumentPositionParams},
    symbol::build_section_tree,
    syntax::{latex, LatexLabelKind, SyntaxNode},
    workspace::DocumentContent,
};
use async_trait::async_trait;
use std::{path::Path, sync::Arc};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexLabelDefinitionProvider;

#[async_trait]
impl FeatureProvider for LatexLabelDefinitionProvider {
    type Params = TextDocumentPositionParams;
    type Output = Vec<LocationLink>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut links = Vec::new();
        if let Some(reference) = Self::find_reference(req) {
            for doc in req.related() {
                let snapshot = Arc::clone(&req.view.snapshot);
                let view = DocumentView::analyze(
                    snapshot,
                    Arc::clone(&doc),
                    &req.options,
                    &req.current_dir,
                );
                Self::find_definitions(
                    &view,
                    &req.options,
                    &req.current_dir,
                    &reference,
                    &mut links,
                );
            }
        }
        links
    }
}

impl LatexLabelDefinitionProvider {
    fn find_reference(req: &FeatureRequest<TextDocumentPositionParams>) -> Option<&latex::Token> {
        if let DocumentContent::Latex(table) = &req.current().content {
            table
                .labels
                .iter()
                .flat_map(|label| label.names(&table))
                .find(|label| label.range().contains(req.params.position))
        } else {
            None
        }
    }

    fn find_definitions(
        view: &DocumentView,
        options: &Options,
        current_dir: &Path,
        reference: &latex::Token,
        links: &mut Vec<LocationLink>,
    ) {
        if let DocumentContent::Latex(table) = &view.current.content {
            let outline = Outline::analyze(view, options, current_dir);
            let section_tree = build_section_tree(view, table, options, current_dir);
            for label in &table.labels {
                if label.kind == LatexLabelKind::Definition {
                    let context = OutlineContext::parse(view, &outline, *label);
                    for name in label.names(&table) {
                        if name.text() == reference.text() {
                            let target_range = if let Some(OutlineContextItem::Section { .. }) =
                                context.as_ref().map(|ctx| &ctx.item)
                            {
                                section_tree
                                    .find(reference.text())
                                    .map(|sec| sec.full_range)
                            } else {
                                context.as_ref().map(|ctx| ctx.range)
                            };

                            links.push(LocationLink {
                                origin_selection_range: Some(reference.range()),
                                target_uri: view.current.uri.clone().into(),
                                target_range: target_range
                                    .unwrap_or_else(|| table[label.parent].range()),
                                target_selection_range: table[label.parent].range(),
                            });
                        }
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
            .test_position(LatexLabelDefinitionProvider)
            .await;

        assert!(actual_links.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_links = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_position(LatexLabelDefinitionProvider)
            .await;

        assert!(actual_links.is_empty());
    }

    #[tokio::test]
    async fn unknown_context() {
        let actual_links = FeatureTester::new()
            .file("foo.tex", r#"\label{foo}"#)
            .file(
                "bar.tex",
                indoc!(
                    r#"
                        \begin{a}\begin{b}\label{foo}\end{b}\end{a}
                        \input{baz.tex}
                    "#
                ),
            )
            .file("baz.tex", r#"\ref{foo}"#)
            .main("baz.tex")
            .position(0, 5)
            .test_position(LatexLabelDefinitionProvider)
            .await;

        let expected_links = vec![LocationLink {
            origin_selection_range: Some(Range::new_simple(0, 5, 0, 8)),
            target_uri: FeatureTester::uri("bar.tex").into(),
            target_range: Range::new_simple(0, 18, 0, 29),
            target_selection_range: Range::new_simple(0, 18, 0, 29),
        }];

        assert_eq!(actual_links, expected_links);
    }
}
