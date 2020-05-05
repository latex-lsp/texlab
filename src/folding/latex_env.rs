use crate::{
    feature::{FeatureProvider, FeatureRequest},
    protocol::{FoldingRange, FoldingRangeKind, FoldingRangeParams},
    syntax::SyntaxNode,
    workspace::DocumentContent,
};
use async_trait::async_trait;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexEnvironmentFoldingProvider;

#[async_trait]
impl FeatureProvider for LatexEnvironmentFoldingProvider {
    type Params = FoldingRangeParams;
    type Output = Vec<FoldingRange>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut foldings = Vec::new();
        if let DocumentContent::Latex(table) = &req.current().content {
            for env in &table.environments {
                let left_node = &table[env.left.parent];
                let right_node = &table[env.right.parent];
                let folding = FoldingRange {
                    start_line: left_node.end().line,
                    start_character: Some(left_node.end().character),
                    end_line: right_node.start().line,
                    end_character: Some(right_node.start().character),
                    kind: Some(FoldingRangeKind::Region),
                };
                foldings.push(folding);
            }
        }
        foldings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;
    use indoc::indoc;

    #[tokio::test]
    async fn multiline() {
        let actual_foldings = FeatureTester::new()
            .file(
                "main.tex",
                indoc!(
                    r#"
                        \begin{foo}
                        \end{foo}
                    "#
                ),
            )
            .main("main.tex")
            .test_folding(LatexEnvironmentFoldingProvider)
            .await;

        let expected_foldings = vec![FoldingRange {
            start_line: 0,
            start_character: Some(11),
            end_line: 1,
            end_character: Some(0),
            kind: Some(FoldingRangeKind::Region),
        }];

        assert_eq!(actual_foldings, expected_foldings);
    }

    #[tokio::test]
    async fn bibtex() {
        let actual_foldings = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .test_folding(LatexEnvironmentFoldingProvider)
            .await;

        assert!(actual_foldings.is_empty());
    }
}
