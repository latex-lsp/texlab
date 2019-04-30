use crate::feature::FeatureRequest;
use crate::range;
use crate::syntax::latex::analysis::environment::*;
use crate::syntax::latex::ast::LatexVisitor;
use crate::syntax::text::SyntaxNode;
use crate::workspace::SyntaxTree;
use lsp_types::{Position, RenameParams, TextEdit, WorkspaceEdit};
use std::collections::HashMap;

pub struct LatexEnvironmentRenameProvider;

impl LatexEnvironmentRenameProvider {
    pub async fn execute(request: &FeatureRequest<RenameParams>) -> Option<WorkspaceEdit> {
        if let SyntaxTree::Latex(tree) = &request.document.tree {
            let mut analyzer = LatexEnvironmentAnalyzer::new();
            analyzer.visit_root(&tree.root);
            for environment in &analyzer.environments {
                if let Some(left_name) = environment.left.name {
                    if let Some(right_name) = environment.right.name {
                        if range::contains(left_name.range(), request.params.position)
                            || range::contains(right_name.range(), request.params.position)
                        {
                            let edits = vec![
                                TextEdit::new(left_name.range(), request.params.new_name.clone()),
                                TextEdit::new(right_name.range(), request.params.new_name.clone()),
                            ];
                            let mut changes = HashMap::new();
                            changes.insert(request.document.uri.clone(), edits);
                            return Some(WorkspaceEdit::new(changes));
                        }
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;
    use crate::workspace::WorkspaceBuilder;
    use futures::executor::block_on;

    #[test]
    fn test() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "\\begin{foo}\n\\end{bar}");
        let request = FeatureTester::new(builder.workspace, uri.clone(), 0, 8, "baz").into();

        let changes = block_on(LatexEnvironmentRenameProvider::execute(&request))
            .unwrap()
            .changes
            .unwrap();

        assert_eq!(1, changes.len());
        assert_eq!(
            vec![
                TextEdit::new(range::create(0, 7, 0, 10), "baz".to_owned()),
                TextEdit::new(range::create(1, 5, 1, 8), "baz".to_owned())
            ],
            *changes.get(&uri).unwrap()
        );
    }

    #[test]
    fn test_command() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "\\begin{foo}\n\\end{bar}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 5, "baz").into();

        let edit = block_on(LatexEnvironmentRenameProvider::execute(&request));

        assert_eq!(None, edit);
    }

    #[test]
    fn test_bibtex() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.bib", "");
        let request = FeatureTester::new(builder.workspace, uri, 0, 0, "baz").into();

        let edit = block_on(LatexEnvironmentRenameProvider::execute(&request));

        assert_eq!(None, edit);
    }
}
