use crate::syntax::*;
use crate::workspace::*;
use futures_boxed::boxed;
use lsp_types::{FoldingRange, FoldingRangeKind, FoldingRangeParams};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexEnvironmentFoldingProvider;

impl FeatureProvider for LatexEnvironmentFoldingProvider {
    type Params = FoldingRangeParams;
    type Output = Vec<FoldingRange>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<FoldingRangeParams>,
    ) -> Vec<FoldingRange> {
        let mut foldings = Vec::new();
        if let SyntaxTree::Latex(tree) = &request.document().tree {
            for environment in &tree.env.environments {
                let start = environment.left.command.end();
                let end = environment.right.command.start();
                foldings.push(FoldingRange {
                    start_line: start.line,
                    start_character: Some(start.character),
                    end_line: end.line,
                    end_character: Some(end.character),
                    kind: Some(FoldingRangeKind::Region),
                })
            }
        }
        foldings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiline() {
        let foldings = test_feature(
            LatexEnvironmentFoldingProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\begin{foo}\n\\end{foo}")],
                main_file: "foo.tex",
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            foldings,
            vec![FoldingRange {
                start_line: 0,
                start_character: Some(11),
                end_line: 1,
                end_character: Some(0),
                kind: Some(FoldingRangeKind::Region),
            }]
        );
    }

    #[test]
    fn test_bibtex() {
        let foldings = test_feature(
            LatexEnvironmentFoldingProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, bar = baz}")],
                main_file: "foo.bib",
                ..FeatureSpec::default()
            },
        );
        assert!(foldings.is_empty());
    }
}
