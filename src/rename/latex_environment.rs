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
    use crate::completion::latex::data::types::LatexComponentDatabase;
    use crate::feature::FeatureSpec;
    use crate::range;
    use crate::test_feature;
    use lsp_types::Position;

    #[test]
    fn test_environment() {
        let edit = test_feature!(
            LatexEnvironmentRenameProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\begin{foo}\n\\end{bar}")],
                main_file: "foo.tex",
                position: Position::new(0, 8),
                new_name: "baz",
                component_database: LatexComponentDatabase::default(),
            }
        );
        let mut changes = HashMap::new();
        changes.insert(
            FeatureSpec::uri("foo.tex"),
            vec![
                TextEdit::new(range::create(0, 7, 0, 10), "baz".to_owned()),
                TextEdit::new(range::create(1, 5, 1, 8), "baz".to_owned()),
            ],
        );
        assert_eq!(edit, Some(WorkspaceEdit::new(changes)));
    }

    #[test]
    fn test_command() {
        let edit = test_feature!(
            LatexEnvironmentRenameProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\begin{foo}\n\\end{bar}")],
                main_file: "foo.tex",
                position: Position::new(0, 5),
                new_name: "baz",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(edit, None);
    }

    #[test]
    fn test_bibtex() {
        let edit = test_feature!(
            LatexEnvironmentRenameProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "")],
                main_file: "foo.bib",
                position: Position::new(0, 0),
                new_name: "baz",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(edit, None);
    }
}
