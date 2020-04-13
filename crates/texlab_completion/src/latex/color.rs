use super::combinators::{self, Parameter};
use crate::factory;
use futures_boxed::boxed;
use texlab_feature::{FeatureProvider, FeatureRequest};
use texlab_protocol::{CompletionItem, CompletionParams, TextEdit};
use texlab_syntax::LANGUAGE_DATA;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexColorCompletionProvider;

impl FeatureProvider for LatexColorCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let parameters = LANGUAGE_DATA.color_commands.iter().map(|cmd| Parameter {
            name: &cmd.name,
            index: cmd.index,
        });

        combinators::argument(req, parameters, |ctx| async move {
            let mut items = Vec::new();
            for name in &LANGUAGE_DATA.colors {
                let text_edit = TextEdit::new(ctx.range, name.into());
                let item = factory::color(req, name, text_edit);
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
    use texlab_protocol::{Range, RangeExt};

    #[tokio::test]
    async fn empty_latex_document() {
        let actual_items = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion(LatexColorCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion(LatexColorCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_color() {
        let actual_items = FeatureTester::new()
            .file("main.tex", r#"\color{}"#)
            .main("main.tex")
            .position(0, 7)
            .test_completion(LatexColorCompletionProvider)
            .await;

        assert!(!actual_items.is_empty());
        assert_eq!(
            actual_items[0]
                .text_edit
                .as_ref()
                .map(|edit| edit.range)
                .unwrap(),
            Range::new_simple(0, 7, 0, 7)
        );
    }

    #[tokio::test]
    async fn inside_define_color_set() {
        let actual_items = FeatureTester::new()
            .file("main.tex", r#"\color{}"#)
            .main("main.tex")
            .position(0, 8)
            .test_completion(LatexColorCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }
}
