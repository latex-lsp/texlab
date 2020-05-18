use super::combinators;
use crate::{
    completion::{Item, ItemData},
    feature::FeatureRequest,
    protocol::{CompletionParams, Range},
    syntax::latex,
    workspace::DocumentContent,
};

pub async fn complete_latex_user_commands<'a>(
    req: &'a FeatureRequest<CompletionParams>,
    items: &mut Vec<Item<'a>>,
) {
    combinators::command(req, |current_cmd_node| async move {
        let current_cmd = req
            .current()
            .content
            .as_latex()
            .unwrap()
            .as_command(current_cmd_node)
            .unwrap();

        for table in req
            .related()
            .into_iter()
            .flat_map(|doc| doc.content.as_latex())
        {
            table
                .commands
                .iter()
                .filter(|cmd_node| **cmd_node != current_cmd_node)
                .map(|cmd_node| {
                    let name = &table.as_command(*cmd_node).unwrap().name.text()[1..];
                    Item::new(
                        current_cmd.short_name_range(),
                        ItemData::UserCommand { name },
                    )
                })
                .for_each(|item| items.push(item));
        }
    })
    .await;
}

pub async fn complete_latex_user_environments<'a>(
    req: &'a FeatureRequest<CompletionParams>,
    items: &mut Vec<Item<'a>>,
) {
    fn make_item(
        table: &latex::SymbolTable,
        delim: latex::EnvironmentDelimiter,
        name_range: Range,
    ) -> Option<Item> {
        delim
            .name(&table)
            .map(|name| Item::new(name_range, ItemData::UserEnvironment { name: &name.text() }))
    }

    combinators::environment(req, |ctx| async move {
        for doc in req.related() {
            if let DocumentContent::Latex(table) = &doc.content {
                for env in &table.environments {
                    if (env.left.parent == ctx.node || env.right.parent == ctx.node)
                        && doc.uri == req.current().uri
                    {
                        continue;
                    }

                    if let Some(item) = make_item(&table, env.left, ctx.range) {
                        items.push(item);
                    }

                    if let Some(item) = make_item(&table, env.right, ctx.range) {
                        items.push(item);
                    }
                }
            }
        }
    })
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;
    use indoc::indoc;
    use itertools::Itertools;

    #[tokio::test]
    async fn empty_latex_document_command() {
        let req = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();
        complete_latex_user_commands(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document_command() {
        let req = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();
        complete_latex_user_commands(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_latex_document_environment() {
        let req = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();
        complete_latex_user_environments(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document_environment() {
        let req = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();
        complete_latex_user_environments(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn command() {
        let req = FeatureTester::new()
            .file(
                "foo.tex",
                indoc!(
                    r#"
                        \include{bar}
                        \foo
                    "#
                ),
            )
            .file("bar.tex", r#"\bar"#)
            .file("baz.tex", r#"\baz"#)
            .main("foo.tex")
            .position(1, 2)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_user_commands(&req, &mut actual_items).await;

        let actual_labels: Vec<_> = actual_items
            .into_iter()
            .map(|item| item.data.label().to_owned())
            .collect();
        assert_eq!(actual_labels, vec!["include", "bar"]);
    }

    #[tokio::test]
    async fn environment() {
        let req = FeatureTester::new()
            .file(
                "foo.tex",
                indoc!(
                    r#"
                        \include{bar}
                        \begin{foo}
                    "#
                ),
            )
            .file("bar.tex", r#"\begin{bar}\end{bar}"#)
            .file("baz.tex", r#"\begin{baz}\end{baz}"#)
            .main("foo.tex")
            .position(1, 9)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_user_environments(&req, &mut actual_items).await;

        let actual_labels: Vec<_> = actual_items
            .into_iter()
            .map(|item| item.data.label().to_owned())
            .unique()
            .collect();
        assert_eq!(actual_labels, vec!["bar"]);
    }
}
