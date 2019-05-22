use crate::feature::FeatureRequest;
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use lsp_types::{FoldingRange, FoldingRangeKind, FoldingRangeParams};

pub struct LatexSectionFoldingProvider;

impl LatexSectionFoldingProvider {
    pub async fn execute(request: &FeatureRequest<FoldingRangeParams>) -> Vec<FoldingRange> {
        let mut foldings = Vec::new();
        if let SyntaxTree::Latex(tree) = &request.document.tree {
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
    use crate::feature::FeatureSpec;
    use crate::test_feature;

    #[test]
    fn test_nesting() {
        let foldings = test_feature!(
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
        let foldings = test_feature!(
            LatexSectionFoldingProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, bar = baz}")],
                main_file: "foo.bib",
                ..FeatureSpec::default()
            }
        );
        assert_eq!(foldings, Vec::new());
    }
}
