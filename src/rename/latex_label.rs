use crate::feature::FeatureRequest;
use crate::range;
use crate::syntax::latex::{LatexLabelAnalyzer, LatexVisitor};
use crate::syntax::text::SyntaxNode;
use crate::workspace::SyntaxTree;
use lsp_types::{RenameParams, TextEdit, WorkspaceEdit};
use std::collections::HashMap;

pub struct LatexLabelRenameProvider;

impl LatexLabelRenameProvider {
    pub async fn execute(request: &FeatureRequest<RenameParams>) -> Option<WorkspaceEdit> {
        let name = Self::find_label(&request)?;
        let mut changes = HashMap::new();
        for document in &request.related_documents {
            if let SyntaxTree::Latex(tree) = &document.tree {
                let mut analyzer = LatexLabelAnalyzer::new();
                analyzer.visit_root(&tree.root);
                let edits = analyzer
                    .labels
                    .iter()
                    .filter(|label| label.name.text() == name)
                    .map(|label| TextEdit::new(label.name.range(), request.params.new_name.clone()))
                    .collect();
                changes.insert(document.uri.clone(), edits);
            }
        }
        Some(WorkspaceEdit::new(changes))
    }

    fn find_label(request: &FeatureRequest<RenameParams>) -> Option<&str> {
        if let SyntaxTree::Latex(tree) = &request.document.tree {
            let mut analyzer = LatexLabelAnalyzer::new();
            analyzer.visit_root(&tree.root);
            analyzer
                .labels
                .iter()
                .find(|label| range::contains(label.name.range(), request.params.position))
                .map(|label| label.name.text())
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
                component_database: LatexComponentDatabase::default(),
            }
        );
        let mut changes = HashMap::new();
        changes.insert(
            FeatureSpec::uri("foo.tex"),
            vec![TextEdit::new(range::create(0, 7, 0, 10), "bar".to_owned())],
        );
        changes.insert(
            FeatureSpec::uri("bar.tex"),
            vec![TextEdit::new(range::create(0, 5, 0, 8), "bar".to_owned())],
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
                component_database: LatexComponentDatabase::default(),
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
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(edit, None);
    }
}
