use super::combinators::{self, ArgumentContext, Parameter};
use crate::{
    completion::factory,
    feature::{DocumentView, FeatureProvider, FeatureRequest},
    outline::{Outline, OutlineContext},
    protocol::{CompletionItem, CompletionParams, RangeExt, TextEdit},
    syntax::{latex, LatexLabelKind, LatexLabelReferenceSource, LANGUAGE_DATA},
    workspace::DocumentContent,
};
use futures_boxed::boxed;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexLabelCompletionProvider;

impl FeatureProvider for LatexLabelCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let parameters = LANGUAGE_DATA
            .label_commands
            .iter()
            .filter(|cmd| cmd.kind.is_reference())
            .map(|cmd| Parameter {
                name: &cmd.name,
                index: cmd.index,
            });

        combinators::argument(req, parameters, |ctx| async move {
            let source = Self::find_source(ctx);
            let mut items = Vec::new();
            for doc in req.related() {
                let snapshot = Arc::clone(&req.view.snapshot);
                let view = DocumentView::analyze(
                    snapshot,
                    Arc::clone(&doc),
                    &req.options,
                    &req.current_dir,
                );
                let outline = Outline::analyze(&view, &req.options, &req.current_dir);

                if let DocumentContent::Latex(table) = &doc.content {
                    for label in table
                        .labels
                        .iter()
                        .filter(|label| label.kind == LatexLabelKind::Definition)
                        .filter(|label| Self::is_included(&table, label, source))
                    {
                        let outline_context = OutlineContext::parse(&view, &outline, *label);
                        for name in label.names(&table.tree) {
                            let text = name.text().to_owned();
                            let text_edit = TextEdit::new(ctx.range, text.clone());
                            let item =
                                factory::label(req, text, text_edit, outline_context.as_ref());
                            items.push(item);
                        }
                    }
                }
            }
            items
        })
        .await
    }
}

impl LatexLabelCompletionProvider {
    fn find_source(ctx: ArgumentContext) -> LatexLabelReferenceSource {
        match LANGUAGE_DATA
            .label_commands
            .iter()
            .find(|cmd| cmd.name == ctx.parameter.name && cmd.index == ctx.parameter.index)
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
        let label_range = table.tree.range(label.parent);
        match source {
            LatexLabelReferenceSource::Everything => true,
            LatexLabelReferenceSource::Math => table
                .environments
                .iter()
                .filter(|env| env.left.is_math(&table.tree))
                .any(|env| env.range(&table.tree).contains_exclusive(label_range.start)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;
    use indoc::indoc;

    #[tokio::test]
    async fn empty_latex_document() {
        let actual_items = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion(LatexLabelCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion(LatexLabelCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_of_ref() {
        let actual_labels: Vec<_> = FeatureTester::new()
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
            .test_completion(LatexLabelCompletionProvider)
            .await
            .into_iter()
            .map(|item| item.label)
            .collect();

        assert_eq!(actual_labels, vec!["foo", "bar"]);
    }

    #[tokio::test]
    async fn outside_of_ref() {
        let actual_items = FeatureTester::new()
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
            .test_completion(LatexLabelCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn eqref() {
        let actual_labels: Vec<_> = FeatureTester::new()
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
            .test_completion(LatexLabelCompletionProvider)
            .await
            .into_iter()
            .map(|item| item.label)
            .collect();

        assert_eq!(actual_labels, vec!["foo"]);
    }
}
