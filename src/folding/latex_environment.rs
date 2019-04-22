use crate::feature::FeatureRequest;
use crate::syntax::latex::analysis::environment::LatexEnvironmentAnalyzer;
use crate::syntax::latex::ast::LatexVisitor;
use crate::syntax::text::SyntaxNode;
use crate::workspace::SyntaxTree;
use lsp_types::{FoldingRange, FoldingRangeKind, FoldingRangeParams};

pub struct LatexEnvironmentFoldingProvider;

impl LatexEnvironmentFoldingProvider {
    pub async fn execute(request: &FeatureRequest<FoldingRangeParams>) -> Vec<FoldingRange> {
        let mut foldings = Vec::new();
        if let SyntaxTree::Latex(tree) = &request.document.tree {
            let mut analyzer = LatexEnvironmentAnalyzer::new();
            analyzer.visit_root(&tree.root);
            for environment in &analyzer.environments {
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
    use crate::feature::FeatureTester;
    use crate::workspace::WorkspaceBuilder;
    use futures::executor;

    #[test]
    fn test_multiline() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "\\begin{foo}\n\\end{foo}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 0, "").into();

        let results = executor::block_on(LatexEnvironmentFoldingProvider::execute(&request));

        let folding = FoldingRange {
            start_line: 0,
            start_character: Some(11),
            end_line: 1,
            end_character: Some(0),
            kind: Some(FoldingRangeKind::Region),
        };
        assert_eq!(vec![folding], results);
    }

    #[test]
    fn test_bibtex() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.bib", "@article{foo, bar = baz}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 0, "").into();

        let results = executor::block_on(LatexEnvironmentFoldingProvider::execute(&request));

        assert_eq!(results, Vec::new());
    }
}
