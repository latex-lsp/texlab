use super::combinators::{self, ArgumentContext, Parameter};
use crate::{
    completion::types::{Item, ItemData},
    feature::{DocumentView, FeatureRequest},
    outline::{Outline, OutlineContext, OutlineContextItem},
    protocol::{CompletionParams, RangeExt},
    syntax::{
        latex, LatexLabelKind, LatexLabelReferenceSource, Structure, SyntaxNode, LANGUAGE_DATA,
    },
    workspace::DocumentContent,
};
use std::sync::Arc;

pub async fn complete_latex_labels<'a>(
    req: &'a FeatureRequest<CompletionParams>,
    items: &mut Vec<Item<'a>>,
) {
    let parameters = LANGUAGE_DATA
        .label_commands
        .iter()
        .filter(|cmd| cmd.kind.is_reference())
        .map(|cmd| Parameter {
            name: &cmd.name[1..],
            index: cmd.index,
        });

    combinators::argument(req, parameters, |ctx| async move {
        let source = find_source(ctx);
        for doc in req.related() {
            let snapshot = Arc::clone(&req.view.snapshot);
            let view =
                DocumentView::analyze(snapshot, Arc::clone(&doc), &req.options, &req.current_dir);
            let outline = Outline::analyze(&view, &req.options, &req.current_dir);

            if let DocumentContent::Latex(table) = &doc.content {
                for label in table
                    .labels
                    .iter()
                    .filter(|label| label.kind == LatexLabelKind::Definition)
                    .filter(|label| is_included(&table, label, source))
                {
                    let outline_ctx = OutlineContext::parse(&view, &outline, *label);

                    let kind = match outline_ctx.as_ref().map(|ctx| &ctx.item) {
                        Some(OutlineContextItem::Section { .. }) => Structure::Section,
                        Some(OutlineContextItem::Caption { .. }) => Structure::Float,
                        Some(OutlineContextItem::Theorem { .. }) => Structure::Theorem,
                        Some(OutlineContextItem::Equation) => Structure::Equation,
                        Some(OutlineContextItem::Item) => Structure::Item,
                        None => Structure::Label,
                    };

                    for name in label.names(&table) {
                        let header = outline_ctx.as_ref().and_then(|ctx| ctx.detail());
                        let footer = outline_ctx.as_ref().and_then(|ctx| match &ctx.item {
                            OutlineContextItem::Caption { text, .. } => Some(text.clone()),
                            _ => None,
                        });

                        let text = outline_ctx
                            .as_ref()
                            .map(|ctx| format!("{} {}", name.text(), ctx.reference()))
                            .unwrap_or_else(|| name.text().into());

                        let item = Item::new(
                            ctx.range,
                            ItemData::Label {
                                name: name.text(),
                                kind,
                                header,
                                footer,
                                text,
                            },
                        );
                        items.push(item);
                    }
                }
            }
        }
    })
    .await;
}

fn find_source(ctx: ArgumentContext) -> LatexLabelReferenceSource {
    match LANGUAGE_DATA
        .label_commands
        .iter()
        .find(|cmd| &cmd.name[1..] == ctx.parameter.name && cmd.index == ctx.parameter.index)
        .map(|cmd| cmd.kind)
        .unwrap()
    {
        LatexLabelKind::Definition => unreachable!(),
        LatexLabelKind::Reference(source) => source,
    }
}

fn is_included(
    table: &latex::SymbolTable,
    label: &latex::Label,
    source: LatexLabelReferenceSource,
) -> bool {
    let label_range = table[label.parent].range();
    match source {
        LatexLabelReferenceSource::Everything => true,
        LatexLabelReferenceSource::Math => table
            .environments
            .iter()
            .filter(|env| env.left.is_math(&table))
            .any(|env| env.range(&table).contains_exclusive(label_range.start)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;
    use indoc::indoc;

    #[tokio::test]
    async fn empty_latex_document() {
        let req = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_labels(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let req = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_labels(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_of_ref() {
        let req = FeatureTester::new()
            .file(
                "foo.tex",
                indoc!(
                    r#"
                        \addbibresource{bar.bib}
                        \include{baz}
                        \ref{}
                    "#
                ),
            )
            .file("bar.bib", "")
            .file("baz.tex", r#"\label{foo}\label{bar}\ref{baz}"#)
            .main("foo.tex")
            .position(2, 5)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_labels(&req, &mut actual_items).await;

        let actual_labels: Vec<_> = actual_items
            .into_iter()
            .map(|item| item.data.label().to_owned())
            .collect();
        assert_eq!(actual_labels, vec!["foo", "bar"]);
    }

    #[tokio::test]
    async fn outside_of_ref() {
        let req = FeatureTester::new()
            .file(
                "foo.tex",
                indoc!(
                    r#"
                        \include{bar}
                        \ref{}
                    "#
                ),
            )
            .file("bar.tex", r#"\label{foo}\label{bar}"#)
            .main("foo.tex")
            .position(1, 6)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_labels(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn eqref() {
        let req = FeatureTester::new()
            .file(
                "main.tex",
                indoc!(
                    r#"
                    \begin{align}\label{foo}\end{align}\label{bar}
                    \eqref{}
                "#
                ),
            )
            .main("main.tex")
            .position(1, 7)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_labels(&req, &mut actual_items).await;

        let actual_labels: Vec<_> = actual_items
            .into_iter()
            .map(|item| item.data.label().to_owned())
            .collect();

        assert_eq!(actual_labels, vec!["foo"]);
    }
}
