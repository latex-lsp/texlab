use crate::feature::{FeatureProvider, FeatureRequest};
use texlab_syntax::*;
use futures_boxed::boxed;
use lsp_types::{FoldingRange, FoldingRangeKind, FoldingRangeParams};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexSectionFoldingProvider;

impl FeatureProvider for LatexSectionFoldingProvider {
    type Params = FoldingRangeParams;
    type Output = Vec<FoldingRange>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<FoldingRangeParams>,
    ) -> Vec<FoldingRange> {
        let mut foldings = Vec::new();
        if let SyntaxTree::Latex(tree) = &request.document().tree {
            let sections = &tree.sections;
            for i in 0..sections.len() {
                let current = &sections[i];
                let mut next = None;
                for j in i + 1..sections.len() {
                    next = Some(&sections[j]);
                    if current.level >= sections[j].level {
                        break;
                    }
                }

                if let Some(next) = next {
                    if next.command.start().line > 0 {
                        let folding = FoldingRange {
                            start_line: current.command.end().line,
                            start_character: Some(current.command.end().character),
                            end_line: next.command.start().line - 1,
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
    use crate::feature::{test_feature, FeatureSpec};

    #[test]
    fn test_nesting() {
        let foldings = test_feature(
            LatexSectionFoldingProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\section{Foo}\nfoo\n\\subsection{Bar}\nbar\n\\section{Baz}\nbaz\n\\section{Qux}")],
                main_file: "foo.tex",
                ..FeatureSpec::default()
            }
        );
        assert_eq!(
            foldings,
            vec![
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
                }
            ]
        );
    }

    #[test]
    fn test_bibtex() {
        let foldings = test_feature(
            LatexSectionFoldingProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, bar = baz}")],
                main_file: "foo.bib",
                ..FeatureSpec::default()
            },
        );
        assert!(foldings.is_empty());
    }
}
