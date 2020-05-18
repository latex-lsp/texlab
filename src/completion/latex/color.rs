use super::combinators::{self, Parameter};
use crate::{
    completion::types::{Item, ItemData},
    feature::FeatureRequest,
    protocol::CompletionParams,
    syntax::LANGUAGE_DATA,
};

pub async fn complete_latex_colors<'a>(
    req: &'a FeatureRequest<CompletionParams>,
    items: &mut Vec<Item<'a>>,
) {
    let parameters = LANGUAGE_DATA.color_commands.iter().map(|cmd| Parameter {
        name: &cmd.name[1..],
        index: cmd.index,
    });

    combinators::argument(req, parameters, |ctx| async move {
        for name in &LANGUAGE_DATA.colors {
            let item = Item::new(ctx.range, ItemData::Color { name });
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
        complete_latex_colors(&req, &mut actual_items).await;

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
        complete_latex_colors(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_color() {
        let req = FeatureTester::new()
            .file("main.tex", r#"\color{}"#)
            .main("main.tex")
            .position(0, 7)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();
        complete_latex_colors(&req, &mut actual_items).await;

        assert!(!actual_items.is_empty());
        assert_eq!(actual_items[0].range, Range::new_simple(0, 7, 0, 7));
    }

    #[tokio::test]
    async fn inside_define_color_set() {
        let req = FeatureTester::new()
            .file("main.tex", r#"\color{}"#)
            .main("main.tex")
            .position(0, 8)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();
        complete_latex_colors(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }
}
