use super::combinators;
use crate::factory::{self, LatexComponentId};
use async_trait::async_trait;
use texlab_feature::{DocumentContent, FeatureProvider, FeatureRequest};
use texlab_protocol::{CompletionItem, CompletionParams, TextEdit};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexTheoremEnvironmentCompletionProvider;

#[async_trait]
impl FeatureProvider for LatexTheoremEnvironmentCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        combinators::environment(req, |ctx| async move {
            let mut items = Vec::new();
            for doc in req.related() {
                if let DocumentContent::Latex(table) = &doc.content {
                    for theorem in &table.theorem_definitions {
                        let name = theorem.name(&table).text().to_owned();
                        let text_edit = TextEdit::new(ctx.range, name.clone());
                        let item =
                            factory::environment(req, name, text_edit, &LatexComponentId::User);
                        items.push(item);
                    }
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
    use indoc::indoc;
    use texlab_feature::FeatureTester;
    use texlab_protocol::{CompletionTextEditExt, Range, RangeExt};

    #[tokio::test]
    async fn empty_latex_document() {
        let actual_items = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion(LatexTheoremEnvironmentCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion(LatexTheoremEnvironmentCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_begin() {
        let actual_items = FeatureTester::new()
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
            .test_completion(LatexTheoremEnvironmentCompletionProvider)
            .await;

        assert_eq!(actual_items.len(), 1);
        assert_eq!(actual_items[0].label, "theorem");
        assert_eq!(
            actual_items[0]
                .text_edit
                .as_ref()
                .and_then(|edit| edit.text_edit())
                .map(|edit| edit.range)
                .unwrap(),
            Range::new_simple(1, 7, 1, 9)
        );
    }

    #[tokio::test]
    async fn outside_begin() {
        let actual_items = FeatureTester::new()
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
            .test_completion(LatexTheoremEnvironmentCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }
}
