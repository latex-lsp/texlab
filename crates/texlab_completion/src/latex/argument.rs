use super::combinators::{self, Parameter};
use crate::factory;
use async_trait::async_trait;
use std::iter;
use texlab_feature::{FeatureProvider, FeatureRequest};
use texlab_protocol::{CompletionItem, CompletionParams, TextEdit};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexArgumentCompletionProvider;

#[async_trait]
impl FeatureProvider for LatexArgumentCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut all_items = Vec::new();
        for comp in req.view.components() {
            for cmd in &comp.commands {
                let name = format!("\\{}", cmd.name);
                for (i, param) in cmd.parameters.iter().enumerate() {
                    let mut items = combinators::argument(
                        req,
                        iter::once(Parameter {
                            name: &name,
                            index: i,
                        }),
                        |ctx| async move {
                            let mut items = Vec::new();
                            for arg in &param.0 {
                                let text_edit = TextEdit::new(ctx.range, (&arg.name).into());
                                let item = factory::argument(
                                    req,
                                    &arg.name,
                                    text_edit,
                                    arg.image.as_ref().map(AsRef::as_ref),
                                );
                                items.push(item);
                            }
                            items
                        },
                    )
                    .await;
                    all_items.append(&mut items);
                }
            }
        }
        all_items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use texlab_feature::FeatureTester;
    use texlab_protocol::{CompletionTextEditExt, Range, RangeExt};

    #[tokio::test]
    async fn empty_latex_document() {
        let actual_items = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion(LatexArgumentCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion(LatexArgumentCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_mathbb_empty() {
        let actual_items = FeatureTester::new()
            .file(
                "main.tex",
                indoc!(
                    r#"
                        \usepackage{amsfonts}
                        \mathbb{}
                    "#
                ),
            )
            .main("main.tex")
            .position(1, 8)
            .test_completion(LatexArgumentCompletionProvider)
            .await;

        assert!(!actual_items.is_empty());
        assert_eq!(
            actual_items[0]
                .text_edit
                .as_ref()
                .and_then(|edit| edit.text_edit())
                .map(|edit| edit.range)
                .unwrap(),
            Range::new_simple(1, 8, 1, 8)
        );
    }

    #[tokio::test]
    async fn inside_mathbb_non_empty() {
        let actual_items = FeatureTester::new()
            .file(
                "main.tex",
                indoc!(
                    r#"
                        \usepackage{amsfonts}
                        \mathbb{foo}
                    "#
                ),
            )
            .main("main.tex")
            .position(1, 8)
            .test_completion(LatexArgumentCompletionProvider)
            .await;

        assert!(!actual_items.is_empty());
        assert_eq!(
            actual_items[0]
                .text_edit
                .as_ref()
                .and_then(|edit| edit.text_edit())
                .map(|edit| edit.range)
                .unwrap(),
            Range::new_simple(1, 8, 1, 11)
        );
    }

    #[tokio::test]
    async fn outside_mathbb_empty() {
        let actual_items = FeatureTester::new()
            .file(
                "main.tex",
                indoc!(
                    r#"
                        \usepackage{amsfonts}
                        \mathbb{}
                    "#
                ),
            )
            .main("main.tex")
            .position(1, 9)
            .test_completion(LatexArgumentCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }
}
