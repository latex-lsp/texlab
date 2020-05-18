use super::combinators::{self, Parameter};
use crate::{
    completion::types::{Item, ItemData},
    feature::FeatureRequest,
    protocol::CompletionParams,
    syntax::LANGUAGE_DATA,
};

const MODEL_NAMES: &[&str] = &["gray", "rgb", "RGB", "HTML", "cmyk"];

pub async fn complete_latex_color_models<'a>(
    req: &'a FeatureRequest<CompletionParams>,
    items: &mut Vec<Item<'a>>,
) {
    let parameters = LANGUAGE_DATA
        .color_model_commands
        .iter()
        .map(|cmd| Parameter {
            name: &cmd.name[1..],
            index: cmd.index,
        });

    combinators::argument(req, parameters, |ctx| async move {
        for name in MODEL_NAMES {
            let item = Item::new(ctx.range, ItemData::ColorModel { name });
            items.push(item);
        }
    })
    .await;
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
        let req = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_color_models(&req, &mut actual_items).await;

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

        complete_latex_color_models(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_define_color() {
        let req = FeatureTester::new()
            .file("main.tex", r#"\definecolor{name}{}"#)
            .main("main.tex")
            .position(0, 19)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_color_models(&req, &mut actual_items).await;

        assert!(!actual_items.is_empty());
        assert_eq!(actual_items[0].range, Range::new_simple(0, 19, 0, 19));
    }

    #[tokio::test]
    async fn inside_define_color_set() {
        let req = FeatureTester::new()
            .file("main.tex", r#"\definecolorset{}"#)
            .main("main.tex")
            .position(0, 16)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_color_models(&req, &mut actual_items).await;

        assert!(!actual_items.is_empty());
        assert_eq!(actual_items[0].range, Range::new_simple(0, 16, 0, 16));
    }
}
