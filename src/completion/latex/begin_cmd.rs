use super::combinators;
use crate::{
    completion::types::{Item, ItemData},
    feature::FeatureRequest,
    protocol::CompletionParams,
};

pub async fn complete_latex_begin_command<'a>(
    req: &'a FeatureRequest<CompletionParams>,
    items: &mut Vec<Item<'a>>,
) {
    combinators::command(req, |cmd_node| async move {
        let table = req.current().content.as_latex().unwrap();
        let cmd = table.as_command(cmd_node).unwrap();
        let range = cmd.short_name_range();
        items.push(Item::new(range, ItemData::BeginCommand));
    })
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;

    #[tokio::test]
    async fn empty_latex_document() {
        let req = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_begin_command(&req, &mut actual_items).await;

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

        complete_latex_begin_command(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn after_backslash() {
        let req = FeatureTester::new()
            .file("main.tex", r#"\"#)
            .main("main.tex")
            .position(0, 1)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_begin_command(&req, &mut actual_items).await;

        assert_eq!(actual_items.len(), 1);
    }
}
