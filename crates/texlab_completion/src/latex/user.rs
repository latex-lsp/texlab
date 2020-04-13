use super::combinators;
use crate::factory::{self, LatexComponentId};
use futures_boxed::boxed;
use itertools::Itertools;
use texlab_feature::{DocumentContent, FeatureProvider, FeatureRequest};
use texlab_protocol::{CompletionItem, CompletionParams, Range, TextEdit};
use texlab_syntax::latex;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexUserCommandCompletionProvider;

impl FeatureProvider for LatexUserCommandCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        combinators::command(req, |current_cmd_node| async move {
            let current_cmd = req
                .current()
                .content
                .as_latex()
                .unwrap()
                .as_command(current_cmd_node)
                .unwrap();

            let mut items = Vec::new();
            for doc in req.related() {
                if let DocumentContent::Latex(table) = &doc.content {
                    table
                        .commands
                        .iter()
                        .filter(|cmd_node| **cmd_node != current_cmd_node)
                        .map(|cmd_node| {
                            let cmd = table.as_command(*cmd_node).unwrap();
                            cmd.name.text()[1..].to_owned()
                        })
                        .unique()
                        .map(|cmd| {
                            let text_edit =
                                TextEdit::new(current_cmd.short_name_range(), cmd.clone());
                            factory::command(
                                req,
                                cmd,
                                None,
                                None,
                                text_edit,
                                &LatexComponentId::User,
                            )
                        })
                        .for_each(|item| items.push(item));
                }
            }
            items
        })
        .await
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexUserEnvironmentCompletionProvider;

impl FeatureProvider for LatexUserEnvironmentCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        combinators::environment(req, |ctx| async move {
            let mut items = Vec::new();
            for doc in req.related() {
                if let DocumentContent::Latex(table) = &doc.content {
                    for env in &table.environments {
                        if (env.left.parent == ctx.node || env.right.parent == ctx.node)
                            && doc.uri == req.current().uri
                        {
                            continue;
                        }

                        if let Some(item) = Self::make_item(req, &table, env.left, ctx.range) {
                            items.push(item);
                        }

                        if let Some(item) = Self::make_item(req, &table, env.right, ctx.range) {
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

impl LatexUserEnvironmentCompletionProvider {
    fn make_item(
        req: &FeatureRequest<CompletionParams>,
        table: &latex::SymbolTable,
        delim: latex::EnvironmentDelimiter,
        name_range: Range,
    ) -> Option<CompletionItem> {
        delim.name(&table).map(|name| {
            let text = name.text().to_owned();
            let text_edit = TextEdit::new(name_range, text.clone());
            factory::environment(req, text, text_edit, &LatexComponentId::User)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use itertools::Itertools;
    use texlab_feature::FeatureTester;

    #[tokio::test]
    async fn empty_latex_document_command() {
        let actual_items = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion(LatexUserCommandCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document_command() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion(LatexUserCommandCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_latex_document_environment() {
        let actual_items = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion(LatexUserEnvironmentCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document_environment() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion(LatexUserEnvironmentCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn command() {
        let actual_labels: Vec<_> = FeatureTester::new()
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
            .test_completion(LatexUserCommandCompletionProvider)
            .await
            .into_iter()
            .map(|item| item.label)
            .collect();

        assert_eq!(actual_labels, vec!["include", "bar"]);
    }

    #[tokio::test]
    async fn environment() {
        let actual_labels: Vec<_> = FeatureTester::new()
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
            .test_completion(LatexUserEnvironmentCompletionProvider)
            .await
            .into_iter()
            .map(|item| item.label)
            .unique()
            .collect();

        assert_eq!(actual_labels, vec!["bar"]);
    }
}
