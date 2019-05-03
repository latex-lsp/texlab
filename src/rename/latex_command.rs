use crate::feature::FeatureRequest;
use crate::range;
use crate::syntax::latex::analysis::command::LatexCommandAnalyzer;
use crate::syntax::latex::ast::LatexVisitor;
use crate::syntax::text::SyntaxNode;
use crate::workspace::SyntaxTree;
use lsp_types::{RenameParams, TextEdit, WorkspaceEdit};
use std::collections::HashMap;

pub struct LatexCommandRenameProvider;

impl LatexCommandRenameProvider {
    pub async fn execute(request: &FeatureRequest<RenameParams>) -> Option<WorkspaceEdit> {
        let name = Self::find_command(&request)?;
        let mut changes = HashMap::new();
        for document in &request.related_documents {
            if let SyntaxTree::Latex(tree) = &document.tree {
                let mut analyzer = LatexCommandAnalyzer::new();
                analyzer.visit_root(&tree.root);
                let edits: Vec<TextEdit> = analyzer
                    .commands
                    .iter()
                    .filter(|command| command.name.text() == name)
                    .map(|command| {
                        TextEdit::new(
                            command.name.range(),
                            format!("\\{}", request.params.new_name),
                        )
                    })
                    .collect();
                changes.insert(document.uri.clone(), edits);
            }
        }
        Some(WorkspaceEdit::new(changes))
    }

    fn find_command(request: &FeatureRequest<RenameParams>) -> Option<&str> {
        if let SyntaxTree::Latex(tree) = &request.document.tree {
            let mut analyzer = LatexCommandAnalyzer::new();
            analyzer.visit_root(&tree.root);
            analyzer
                .commands
                .iter()
                .find(|command| range::contains(command.name.range(), request.params.position))
                .map(|command| command.name.text())
        } else {
            None
        }
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
    fn test() {
        let edit = test_feature!(
            LatexCommandRenameProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\include{bar.tex}\n\\baz"),
                    FeatureSpec::file("bar.tex", "\\baz"),
                ],
                main_file: "foo.tex",
                position: Position::new(1, 2),
                new_name: "qux",
                component_database: LatexComponentDatabase::default(),
            }
        );
        let mut changes = HashMap::new();
        changes.insert(
            FeatureSpec::uri("foo.tex"),
            vec![TextEdit::new(range::create(1, 0, 1, 4), "\\qux".to_owned())],
        );
        changes.insert(
            FeatureSpec::uri("bar.tex"),
            vec![TextEdit::new(range::create(0, 0, 0, 4), "\\qux".to_owned())],
        );
        assert_eq!(edit, Some(WorkspaceEdit::new(changes)));
    }

    #[test]
    fn test_bibtex() {
        let edit = test_feature!(
            LatexCommandRenameProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, bar = baz}")],
                main_file: "foo.bib",
                position: Position::new(0, 14),
                new_name: "qux",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(edit, None);
    }
}
