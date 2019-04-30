use crate::feature::FeatureRequest;
use crate::syntax::latex::analysis::section::LatexSectionAnalyzer;
use crate::syntax::latex::ast::LatexVisitor;
use crate::syntax::text::SyntaxNode;
use crate::workspace::SyntaxTree;
use lsp_types::{FoldingRange, FoldingRangeKind, FoldingRangeParams};

pub struct LatexSectionFoldingProvider;

impl LatexSectionFoldingProvider {
    pub async fn execute(request: &FeatureRequest<FoldingRangeParams>) -> Vec<FoldingRange> {
        let mut foldings = Vec::new();
        if let SyntaxTree::Latex(tree) = &request.document.tree {
            let mut analyzer = LatexSectionAnalyzer::new();
            analyzer.visit_root(&tree.root);
            let sections = analyzer.sections;
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
        foldings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;
    use crate::workspace::WorkspaceBuilder;
    use futures::executor::block_on;

    #[test]
    fn test_nesting() {
        let text =
            "\\section{Foo}\nfoo\n\\subsection{Bar}\nbar\n\\section{Baz}\nbaz\n\\section{Qux}";

        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", text);
        let request = FeatureTester::new(builder.workspace, uri, 0, 0, "").into();

        let results = block_on(LatexSectionFoldingProvider::execute(&request));

        let folding1 = FoldingRange {
            start_line: 0,
            start_character: Some(13),
            end_line: 3,
            end_character: Some(0),
            kind: Some(FoldingRangeKind::Region),
        };
        let folding2 = FoldingRange {
            start_line: 2,
            start_character: Some(16),
            end_line: 3,
            end_character: Some(0),
            kind: Some(FoldingRangeKind::Region),
        };
        let folding3 = FoldingRange {
            start_line: 4,
            start_character: Some(13),
            end_line: 5,
            end_character: Some(0),
            kind: Some(FoldingRangeKind::Region),
        };
        assert_eq!(vec![folding1, folding2, folding3], results)
    }

    #[test]
    fn test_bibtex() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.bib", "@article{foo, bar = baz}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 0, "").into();

        let results = block_on(LatexSectionFoldingProvider::execute(&request));

        assert_eq!(results, Vec::new());
    }
}
