use super::combinators;
use crate::{
    completion::types::{Item, ItemData},
    feature::FeatureRequest,
    protocol::CompletionParams,
};

pub async fn complete_latex_component_commands<'a>(
    req: &'a FeatureRequest<CompletionParams>,
    items: &mut Vec<Item<'a>>,
) {
    combinators::command(req, |cmd_node| async move {
        let table = req.current().content.as_latex().unwrap();
        let cmd = table.as_command(cmd_node).unwrap();
        let range = cmd.short_name_range();

        for comp in req.view.components() {
            for cmd in &comp.commands {
                items.push(Item::new(
                    range,
                    ItemData::ComponentCommand {
                        name: &cmd.name,
                        image: cmd.image.as_deref(),
                        glyph: cmd.glyph.as_deref(),
                        file_names: &comp.file_names,
                    },
                ));
            }
        }
    })
    .await;
}

pub async fn complete_latex_component_environments<'a>(
    req: &'a FeatureRequest<CompletionParams>,
    items: &mut Vec<Item<'a>>,
) {
    combinators::environment(req, |ctx| async move {
        for comp in req.view.components() {
            for env in &comp.environments {
                items.push(Item::new(
                    ctx.range,
                    ItemData::ComponentEnvironment {
                        name: env,
                        file_names: &comp.file_names,
                    },
                ));
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
    async fn empty_latex_document_command() {
        let req = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_component_commands(&req, &mut actual_items).await;

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

        complete_latex_component_commands(&req, &mut actual_items).await;

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

        complete_latex_component_environments(&req, &mut actual_items).await;

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

        complete_latex_component_environments(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn command_start() {
        let req = FeatureTester::new()
            .file("main.tex", r#"\use"#)
            .main("main.tex")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_component_commands(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn command_end() {
        let req = FeatureTester::new()
            .file("main.tex", r#"\use"#)
            .main("main.tex")
            .position(0, 4)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_component_commands(&req, &mut actual_items).await;

        assert!(!actual_items.is_empty());
        assert_eq!(actual_items[0].range, Range::new_simple(0, 1, 0, 4));
    }

    #[tokio::test]
    async fn command_word() {
        let req = FeatureTester::new()
            .file("main.tex", r#"use"#)
            .main("main.tex")
            .position(0, 2)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_component_commands(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn command_package() {
        let req = FeatureTester::new()
            .file(
                "main.tex",
                indoc!(
                    r#"
                        \usepackage{lipsum}
                        \lips
                    "#
                ),
            )
            .main("main.tex")
            .position(1, 2)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_component_commands(&req, &mut actual_items).await;

        assert!(actual_items
            .iter()
            .any(|item| item.data.label() == "lipsum"));
    }

    #[tokio::test]
    async fn command_package_comma_separated() {
        let req = FeatureTester::new()
            .file(
                "main.tex",
                indoc!(
                    r#"
                        \usepackage{geometry, lipsum}
                        \lips
                    "#
                ),
            )
            .main("main.tex")
            .position(1, 2)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_component_commands(&req, &mut actual_items).await;

        assert!(actual_items
            .iter()
            .any(|item| item.data.label() == "lipsum"));
    }

    #[tokio::test]
    async fn command_class() {
        let req = FeatureTester::new()
            .file(
                "main.tex",
                indoc!(
                    r#"
                        \documentclass{book}
                        \chap
                    "#
                ),
            )
            .main("main.tex")
            .position(1, 2)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_component_commands(&req, &mut actual_items).await;

        assert!(actual_items
            .iter()
            .any(|item| item.data.label() == "chapter"));
    }

    #[tokio::test]
    async fn environment_inside_of_empty_begin() {
        let req = FeatureTester::new()
            .file("main.tex", r#"\begin{}"#)
            .main("main.tex")
            .position(0, 7)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_component_environments(&req, &mut actual_items).await;

        assert!(!actual_items.is_empty());
        assert_eq!(actual_items[0].range, Range::new_simple(0, 7, 0, 7));
    }

    #[tokio::test]
    async fn environment_inside_of_non_empty_end() {
        let req = FeatureTester::new()
            .file("main.tex", r#"\end{foo}"#)
            .main("main.tex")
            .position(0, 6)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_component_environments(&req, &mut actual_items).await;

        assert!(!actual_items.is_empty());
        assert_eq!(actual_items[0].range, Range::new_simple(0, 5, 0, 8));
    }

    #[tokio::test]
    async fn environment_outside_of_empty_begin() {
        let req = FeatureTester::new()
            .file("main.tex", r#"\begin{}"#)
            .main("main.tex")
            .position(0, 6)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_component_environments(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn environment_outside_of_empty_end() {
        let req = FeatureTester::new()
            .file("main.tex", r#"\end{}"#)
            .main("main.tex")
            .position(0, 6)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_component_environments(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn environment_inside_of_other_command() {
        let req = FeatureTester::new()
            .file("main.tex", r#"\foo{bar}"#)
            .main("main.tex")
            .position(0, 6)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_component_environments(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn environment_inside_second_argument() {
        let req = FeatureTester::new()
            .file("main.tex", r#"\begin{foo}{bar}"#)
            .main("main.tex")
            .position(0, 14)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_component_environments(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn environment_unterminated() {
        let req = FeatureTester::new()
            .file("main.tex", r#"\begin{foo"#)
            .main("main.tex")
            .position(0, 7)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_component_environments(&req, &mut actual_items).await;

        assert!(!actual_items.is_empty());
        assert_eq!(actual_items[0].range, Range::new_simple(0, 7, 0, 10));
    }
}
