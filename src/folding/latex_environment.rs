use crate::feature::FeatureRequest;
use crate::syntax::latex::{LatexEnvironmentAnalyzer, LatexVisitor};
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
    use crate::completion::latex::data::types::LatexComponentDatabase;
    use crate::feature::FeatureSpec;
    use crate::test_feature;
    use lsp_types::Position;

    #[test]
    fn test_multiline() {
        let foldings = test_feature!(
            LatexEnvironmentFoldingProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\begin{foo}\n\\end{foo}")],
                main_file: "foo.tex",
                position: Position::default(),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
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
        let foldings = test_feature!(
            LatexEnvironmentFoldingProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, bar = baz}")],
                main_file: "foo.bib",
                position: Position::default(),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(foldings, Vec::new());
    }
}
