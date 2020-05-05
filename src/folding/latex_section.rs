use crate::{
    feature::{FeatureProvider, FeatureRequest},
    protocol::{FoldingRange, FoldingRangeKind, FoldingRangeParams},
    syntax::SyntaxNode,
    workspace::DocumentContent,
};
use async_trait::async_trait;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexSectionFoldingProvider;

#[async_trait]
impl FeatureProvider for LatexSectionFoldingProvider {
    type Params = FoldingRangeParams;
    type Output = Vec<FoldingRange>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut foldings = Vec::new();
        if let DocumentContent::Latex(table) = &req.current().content {
            let sections = &table.sections;
            for i in 0..sections.len() {
                let current = &sections[i];
                if let Some(next) = sections
                    .iter()
                    .skip(i + 1)
                    .find(|sec| current.level >= sec.level)
                {
                    let next_node = &table[next.parent];
                    if next_node.start().line > 0 {
                        let current_node = &table[current.parent];
                        let folding = FoldingRange {
                            start_line: current_node.end().line,
                            start_character: Some(current_node.end().character),
                            end_line: next_node.start().line - 1,
                            end_character: Some(0),
                            kind: Some(FoldingRangeKind::Region),
                        };
                        foldings.push(folding);
                    }
                }
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
    async fn nested() {
        let actual_foldings = FeatureTester::new()
            .file(
                "main.tex",
                indoc!(
                    r#"
                        \section{Foo}
                        foo
                        \subsection{Bar}
                        bar
                        \section{Baz}
                        baz
                        \section{Qux}
                    "#
                ),
            )
            .main("main.tex")
            .test_folding(LatexSectionFoldingProvider)
            .await;

        let expected_foldings = vec![
            FoldingRange {
                start_line: 0,
                start_character: Some(13),
                end_line: 3,
                end_character: Some(0),
                kind: Some(FoldingRangeKind::Region),
            },
            FoldingRange {
                start_line: 2,
                start_character: Some(16),
                end_line: 3,
                end_character: Some(0),
                kind: Some(FoldingRangeKind::Region),
            },
            FoldingRange {
                start_line: 4,
                start_character: Some(13),
                end_line: 5,
                end_character: Some(0),
                kind: Some(FoldingRangeKind::Region),
            },
        ];

        assert_eq!(actual_foldings, expected_foldings);
    }

    #[tokio::test]
    async fn bibtex() {
        let actual_foldings = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .test_folding(LatexSectionFoldingProvider)
            .await;

        assert!(actual_foldings.is_empty());
    }
}
