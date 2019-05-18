use crate::feature::FeatureRequest;
use crate::syntax::latex::{LatexCommandAnalyzer, LatexVisitor};
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use lsp_types::{RenameParams, TextEdit, WorkspaceEdit};
use std::borrow::Cow;
use std::collections::HashMap;

pub struct LatexCommandRenameProvider;

impl LatexCommandRenameProvider {
    pub async fn execute(request: &FeatureRequest<RenameParams>) -> Option<WorkspaceEdit> {
        let name = Self::find_command(&request)?;
        let mut changes = HashMap::new();
        for document in &request.related_documents {
            if let SyntaxTree::Latex(tree) = &document.tree {
                let edits: Vec<TextEdit> = tree
                    .commands
                    .iter()
                    .filter(|command| command.name.text() == name)
                    .map(|command| {
                        TextEdit::new(
                            command.name.range(),
                            Cow::from(format!("\\{}", request.params.new_name)),
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
            tree.commands
                .iter()
                .find(|command| command.name.range().contains(request.params.position))
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
    use crate::test_feature;
    use lsp_types::{Position, Range};

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
            vec![TextEdit::new(
                Range::new_simple(1, 0, 1, 4),
                Cow::from("\\qux"),
            )],
        );
        changes.insert(
            FeatureSpec::uri("bar.tex"),
            vec![TextEdit::new(
                Range::new_simple(0, 0, 0, 4),
                Cow::from("\\qux"),
            )],
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
