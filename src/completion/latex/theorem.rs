use super::combinators;
use crate::{
    completion::types::{Item, ItemData},
    feature::FeatureRequest,
    protocol::CompletionParams,
};

pub async fn complete_latex_theorem_environments<'a>(
    req: &'a FeatureRequest<CompletionParams>,
    items: &mut Vec<Item<'a>>,
) {
    combinators::environment(req, |ctx| async move {
        for table in req
            .related()
            .into_iter()
            .filter_map(|doc| doc.content.as_latex())
        {
            for theorem in &table.theorem_definitions {
                let name = theorem.name(&table).text();
                let data = ItemData::UserEnvironment { name };
                let item = Item::new(ctx.range, data);
                items.push(item);
            }
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

        complete_latex_theorem_environments(&req, &mut actual_items).await;

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

        complete_latex_theorem_environments(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_begin() {
        let req = FeatureTester::new()
            .file(
                "main.tex",
                indoc!(
                    r#"
                        \newtheorem{theorem}{Theorem}
                        \begin{th}
                    "#
                ),
            )
            .main("main.tex")
            .position(1, 8)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_theorem_environments(&req, &mut actual_items).await;

        assert_eq!(actual_items.len(), 1);
        assert_eq!(actual_items[0].data.label(), "theorem");
        assert_eq!(actual_items[0].range, Range::new_simple(1, 7, 1, 9));
    }

    #[tokio::test]
    async fn outside_begin() {
        let req = FeatureTester::new()
            .file(
                "main.tex",
                indoc!(
                    r#"
                        \newtheorem{theorem}{Theorem}
                        \begin{th}
                    "#
                ),
            )
            .main("main.tex")
            .position(1, 10)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_theorem_environments(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }
}
