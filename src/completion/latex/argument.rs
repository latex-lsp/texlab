use super::combinators::{self, Parameter};
use crate::{
    completion::types::{Item, ItemData},
    feature::FeatureRequest,
    protocol::CompletionParams,
};
use std::iter;

pub async fn complete_latex_arguments<'a>(
    req: &'a FeatureRequest<CompletionParams>,
    items: &mut Vec<Item<'a>>,
) {
    for comp in req.view.components() {
        for cmd in &comp.commands {
            for (i, param) in cmd.parameters.iter().enumerate() {
                complete_internal(req, items, &cmd.name, i, param).await;
            }
        }
    }
}

async fn complete_internal<'a>(
    req: &'a FeatureRequest<CompletionParams>,
    items: &mut Vec<Item<'a>>,
    name: &'a str,
    index: usize,
    param: &'a crate::components::Parameter,
) {
    combinators::argument(
        req,
        iter::once(Parameter { name, index }),
        |ctx| async move {
            for arg in &param.0 {
                let item = Item::new(
                    ctx.range,
                    ItemData::Argument {
                        name: &arg.name,
                        image: arg.image.as_deref(),
                    },
                );
                items.push(item);
            }
        },
    )
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        feature::FeatureTester,
        protocol::{Range, RangeExt},
    };
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

        complete_latex_arguments(&req, &mut actual_items).await;

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

        complete_latex_arguments(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_mathbb_empty() {
        let req = FeatureTester::new()
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
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_arguments(&req, &mut actual_items).await;

        assert!(!actual_items.is_empty());
        assert_eq!(actual_items[0].range, Range::new_simple(1, 8, 1, 8));
    }

    #[tokio::test]
    async fn inside_mathbb_non_empty() {
        let req = FeatureTester::new()
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
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_arguments(&req, &mut actual_items).await;

        assert!(!actual_items.is_empty());
        assert_eq!(actual_items[0].range, Range::new_simple(1, 8, 1, 11));
    }

    #[tokio::test]
    async fn outside_mathbb_empty() {
        let req = FeatureTester::new()
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
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_arguments(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }
}

/*
use super::combinators::{self, Parameter};
use crate::{
    completion::factory,
    feature::{FeatureProvider, FeatureRequest},
    protocol::{CompletionItem, CompletionParams, TextEdit},
};
use async_trait::async_trait;
use std::iter;

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



*/
