use super::combinators;
use crate::factory::{self, LatexComponentId};
use async_trait::async_trait;
use texlab_feature::{FeatureProvider, FeatureRequest};
use texlab_protocol::{CompletionItem, CompletionParams};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexBeginCommandCompletionProvider;

#[async_trait]
impl FeatureProvider for LatexBeginCommandCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        combinators::command(req, |_| async move {
            let snippet = factory::command_snippet(
                req,
                "begin",
                None,
                "begin{$1}\n\t$0\n\\end{$1}",
                &LatexComponentId::kernel(),
            );
            vec![snippet]
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use texlab_feature::FeatureTester;

    #[tokio::test]
    async fn empty_latex_document() {
        let actual_items = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion(LatexBeginCommandCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion(LatexBeginCommandCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn after_backslash() {
        let actual_items = FeatureTester::new()
            .file("main.tex", r#"\"#)
            .main("main.tex")
            .position(0, 1)
            .test_completion(LatexBeginCommandCompletionProvider)
            .await;

        assert_eq!(actual_items.len(), 1);
    }
}
