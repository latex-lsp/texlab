use crate::feature::FeatureRequest;
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use lsp_types::{RenameParams, TextEdit, WorkspaceEdit};
use std::borrow::Cow;
use std::collections::HashMap;

pub struct LatexLabelRenameProvider;

impl LatexLabelRenameProvider {
    pub async fn execute(request: &FeatureRequest<RenameParams>) -> Option<WorkspaceEdit> {
        let name = Self::find_label(&request)?;
        let mut changes = HashMap::new();
        for document in &request.related_documents {
            if let SyntaxTree::Latex(tree) = &document.tree {
                let edits = tree
                    .labels
                    .iter()
                    .filter(|label| label.name().text() == name)
                    .map(|label| {
                        TextEdit::new(
                            label.name().range(),
                            Cow::from(request.params.new_name.clone()),
                        )
                    })
                    .collect();
                changes.insert(document.uri.clone(), edits);
            }
        }
        Some(WorkspaceEdit::new(changes))
    }

    fn find_label(request: &FeatureRequest<RenameParams>) -> Option<&str> {
        if let SyntaxTree::Latex(tree) = &request.document.tree {
            tree.labels
                .iter()
                .find(|label| label.name().range().contains(request.params.position))
                .map(|label| label.name().text())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureSpec;
    use crate::test_feature;
    use lsp_types::{Position, Range};

    #[test]
    fn test_label() {
        let edit = test_feature!(
            LatexLabelRenameProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\label{foo}\n\\include{bar}"),
                    FeatureSpec::file("bar.tex", "\\ref{foo}"),
                    FeatureSpec::file("baz.tex", "\\ref{foo}"),
                ],
                main_file: "foo.tex",
                position: Position::new(0, 7),
                new_name: "bar",
                ..FeatureSpec::default()
            }
        );
        let mut changes = HashMap::new();
        changes.insert(
            FeatureSpec::uri("foo.tex"),
            vec![TextEdit::new(
                Range::new_simple(0, 7, 0, 10),
                Cow::from("bar"),
            )],
        );
        changes.insert(
            FeatureSpec::uri("bar.tex"),
            vec![TextEdit::new(
                Range::new_simple(0, 5, 0, 8),
                Cow::from("bar"),
            )],
        );
        assert_eq!(edit, Some(WorkspaceEdit::new(changes)));
    }

    #[test]
    fn test_command_args() {
        let edit = test_feature!(
            LatexLabelRenameProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\foo{bar}")],
                main_file: "foo.tex",
                position: Position::new(0, 5),
                new_name: "baz",
                ..FeatureSpec::default()
            }
        );
        assert_eq!(edit, None);
    }

    #[test]
    fn test_bibtex() {
        let edit = test_feature!(
            LatexLabelRenameProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "")],
                main_file: "foo.bib",
                position: Position::new(0, 0),
                new_name: "baz",
                ..FeatureSpec::default()
            }
        );
        assert_eq!(edit, None);
    }
}
