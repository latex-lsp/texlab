use super::combinators::{self, Parameter};
use crate::factory;
use async_trait::async_trait;
use texlab_feature::{FeatureProvider, FeatureRequest};
use texlab_protocol::{CompletionItem, CompletionParams, TextEdit};
use texlab_syntax::LANGUAGE_DATA;

const MODEL_NAMES: &[&str] = &["gray", "rgb", "RGB", "HTML", "cmyk"];

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexColorModelCompletionProvider;

#[async_trait]
impl FeatureProvider for LatexColorModelCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let parameters = LANGUAGE_DATA
            .color_model_commands
            .iter()
            .map(|cmd| Parameter {
                name: &cmd.name,
                index: cmd.index,
            });

        combinators::argument(req, parameters, |ctx| async move {
            let mut items = Vec::new();
            for name in MODEL_NAMES {
                let text_edit = TextEdit::new(ctx.range, (*name).into());
                let item = factory::color_model(req, name, text_edit);
                items.push(item);
            }
            items
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use texlab_feature::FeatureTester;
    use texlab_protocol::{CompletionTextEditExt, Range, RangeExt};

    #[tokio::test]
    async fn empty_latex_document() {
        let actual_items = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion(LatexColorModelCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion(LatexColorModelCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_define_color() {
        let actual_items = FeatureTester::new()
            .file("main.tex", r#"\definecolor{name}{}"#)
            .main("main.tex")
            .position(0, 19)
            .test_completion(LatexColorModelCompletionProvider)
            .await;

        assert!(!actual_items.is_empty());
        assert_eq!(
            actual_items[0]
                .text_edit
                .as_ref()
                .and_then(|edit| edit.text_edit())
                .map(|edit| edit.range)
                .unwrap(),
            Range::new_simple(0, 19, 0, 19)
        );
    }

    #[tokio::test]
    async fn inside_define_color_set() {
        let actual_items = FeatureTester::new()
            .file("main.tex", r#"\definecolorset{}"#)
            .main("main.tex")
            .position(0, 16)
            .test_completion(LatexColorModelCompletionProvider)
            .await;

        assert!(!actual_items.is_empty());
        assert_eq!(
            actual_items[0]
                .text_edit
                .as_ref()
                .and_then(|edit| edit.text_edit())
                .map(|edit| edit.range)
                .unwrap(),
            Range::new_simple(0, 16, 0, 16)
        );
    }
}
