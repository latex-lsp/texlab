use crate::feature::{FeatureProvider, FeatureRequest};
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use futures::prelude::*;
use futures_boxed::boxed;
use lsp_types::{RenameParams, TextEdit, WorkspaceEdit};
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexCommandRenameProvider;

impl FeatureProvider for LatexCommandRenameProvider {
    type Params = RenameParams;
    type Output = Option<WorkspaceEdit>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<RenameParams>,
    ) -> Option<WorkspaceEdit> {
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
}

impl LatexCommandRenameProvider {
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
    use crate::feature::{test_feature, FeatureSpec};
    use lsp_types::{Position, Range};

    #[test]
    fn test() {
        let edit = test_feature(
            LatexCommandRenameProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\include{bar.tex}\n\\baz"),
                    FeatureSpec::file("bar.tex", "\\baz"),
                ],
                main_file: "foo.tex",
                position: Position::new(1, 2),
                new_name: "qux",
                ..FeatureSpec::default()
            },
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
        let edit = test_feature(
            LatexCommandRenameProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, bar = baz}")],
                main_file: "foo.bib",
                position: Position::new(0, 14),
                new_name: "qux",
                ..FeatureSpec::default()
            },
        );
        assert_eq!(edit, None);
    }
}
