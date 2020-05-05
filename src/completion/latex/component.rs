use super::combinators;
use crate::{
    completion::{
        factory::{self, LatexComponentId},
        quality::QualityEvaluator,
        COMPLETION_LIMIT,
    },
    feature::{FeatureProvider, FeatureRequest},
    protocol::{CompletionItem, CompletionParams, TextEdit},
};
use async_trait::async_trait;

#[derive(Debug)]
struct CommandItem {
    id: LatexComponentId,
    name: &'static str,
    image: &'static Option<String>,
    glyph: &'static Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexComponentCommandCompletionProvider;

#[async_trait]
impl FeatureProvider for LatexComponentCommandCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        combinators::command(req, |cmd_node| async move {
            let table = req.current().content.as_latex().unwrap();
            let cmd = table.as_command(cmd_node).unwrap();
            let range = cmd.short_name_range();
            let pos = req.params.text_document_position.position;
            let eval = QualityEvaluator::parse(req.current(), pos);

            let mut items: Vec<_> = req
                .view
                .components()
                .into_iter()
                .flat_map(|comp| {
                    comp.commands.iter().map(move |cmd| CommandItem {
                        id: LatexComponentId::Component(&comp.file_names),
                        name: &cmd.name,
                        image: &cmd.image,
                        glyph: &cmd.glyph,
                    })
                })
                .collect();

            items.sort_by_key(|item| -eval.quality_of(item.name, &None));
            items
                .into_iter()
                .take(COMPLETION_LIMIT)
                .map(|item| {
                    let text_edit = TextEdit::new(range, item.name.into());
                    let new_item = factory::command(
                        req,
                        item.name.into(),
                        item.image.as_ref().map(AsRef::as_ref),
                        item.glyph.as_ref().map(AsRef::as_ref),
                        text_edit,
                        &item.id,
                    );
                    new_item
                })
                .collect()
        })
        .await
    }
}

pub struct LatexComponentEnvironmentCompletionProvider;

#[async_trait]
impl FeatureProvider for LatexComponentEnvironmentCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        combinators::environment(req, |ctx| async move {
            let mut items = Vec::new();
            for comp in req.view.components() {
                let id = LatexComponentId::Component(&comp.file_names);
                for env in &comp.environments {
                    let text_edit = TextEdit::new(ctx.range, env.into());
                    let item = factory::environment(req, env.into(), text_edit, &id);
                    items.push(item);
                }
            }
            items
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        feature::FeatureTester,
        protocol::{CompletionTextEditExt, Range, RangeExt},
    };
    use indoc::indoc;

    #[tokio::test]
    async fn empty_latex_document_command() {
        let actual_items = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion(LatexComponentCommandCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document_command() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion(LatexComponentCommandCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_latex_document_environment() {
        let actual_items = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion(LatexComponentEnvironmentCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document_environment() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion(LatexComponentEnvironmentCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn command_start() {
        let actual_items = FeatureTester::new()
            .file("main.tex", r#"\use"#)
            .main("main.tex")
            .position(0, 0)
            .test_completion(LatexComponentCommandCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn command_end() {
        let actual_items = FeatureTester::new()
            .file("main.tex", r#"\use"#)
            .main("main.tex")
            .position(0, 4)
            .test_completion(LatexComponentCommandCompletionProvider)
            .await;

        assert!(!actual_items.is_empty());
        assert_eq!(
            actual_items[0]
                .text_edit
                .as_ref()
                .and_then(|edit| edit.text_edit())
                .map(|edit| edit.range)
                .unwrap(),
            Range::new_simple(0, 1, 0, 4)
        );
    }

    #[tokio::test]
    async fn command_word() {
        let actual_items = FeatureTester::new()
            .file("main.tex", r#"use"#)
            .main("main.tex")
            .position(0, 2)
            .test_completion(LatexComponentCommandCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn command_package() {
        let actual_items = FeatureTester::new()
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
            .test_completion(LatexComponentCommandCompletionProvider)
            .await;

        assert!(actual_items.iter().any(|item| item.label == "lipsum"));
    }

    #[tokio::test]
    async fn command_package_comma_separated() {
        let actual_items = FeatureTester::new()
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
            .test_completion(LatexComponentCommandCompletionProvider)
            .await;

        assert!(actual_items.iter().any(|item| item.label == "lipsum"));
    }

    #[tokio::test]
    async fn command_class() {
        let actual_items = FeatureTester::new()
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
            .test_completion(LatexComponentCommandCompletionProvider)
            .await;

        assert!(actual_items.iter().any(|item| item.label == "chapter"));
    }

    #[tokio::test]
    async fn environment_inside_of_empty_begin() {
        let actual_items = FeatureTester::new()
            .file("main.tex", r#"\begin{}"#)
            .main("main.tex")
            .position(0, 7)
            .test_completion(LatexComponentEnvironmentCompletionProvider)
            .await;

        assert!(!actual_items.is_empty());
        assert_eq!(
            actual_items[0]
                .text_edit
                .as_ref()
                .and_then(|edit| edit.text_edit())
                .map(|edit| edit.range)
                .unwrap(),
            Range::new_simple(0, 7, 0, 7)
        );
    }

    #[tokio::test]
    async fn environment_inside_of_non_empty_end() {
        let actual_items = FeatureTester::new()
            .file("main.tex", r#"\end{foo}"#)
            .main("main.tex")
            .position(0, 6)
            .test_completion(LatexComponentEnvironmentCompletionProvider)
            .await;

        assert!(!actual_items.is_empty());
        assert_eq!(
            actual_items[0]
                .text_edit
                .as_ref()
                .and_then(|edit| edit.text_edit())
                .map(|edit| edit.range)
                .unwrap(),
            Range::new_simple(0, 5, 0, 8)
        );
    }

    #[tokio::test]
    async fn environment_outside_of_empty_begin() {
        let actual_items = FeatureTester::new()
            .file("main.tex", r#"\begin{}"#)
            .main("main.tex")
            .position(0, 6)
            .test_completion(LatexComponentEnvironmentCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn environment_outside_of_empty_end() {
        let actual_items = FeatureTester::new()
            .file("main.tex", r#"\end{}"#)
            .main("main.tex")
            .position(0, 6)
            .test_completion(LatexComponentEnvironmentCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn environment_inside_of_other_command() {
        let actual_items = FeatureTester::new()
            .file("main.tex", r#"\foo{bar}"#)
            .main("main.tex")
            .position(0, 6)
            .test_completion(LatexComponentEnvironmentCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn environment_inside_second_argument() {
        let actual_items = FeatureTester::new()
            .file("main.tex", r#"\begin{foo}{bar}"#)
            .main("main.tex")
            .position(0, 14)
            .test_completion(LatexComponentEnvironmentCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn environment_unterminated() {
        let actual_items = FeatureTester::new()
            .file("main.tex", r#"\begin{foo"#)
            .main("main.tex")
            .position(0, 7)
            .test_completion(LatexComponentEnvironmentCompletionProvider)
            .await;

        assert!(!actual_items.is_empty());
        assert_eq!(
            actual_items[0]
                .text_edit
                .as_ref()
                .and_then(|edit| edit.text_edit())
                .map(|edit| edit.range)
                .unwrap(),
            Range::new_simple(0, 7, 0, 10)
        );
    }
}
