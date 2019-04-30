use crate::feature::FeatureRequest;
use crate::range;
use crate::syntax::latex::analysis::label::LatexLabelAnalyzer;
use crate::syntax::latex::ast::LatexVisitor;
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
    use crate::feature::FeatureTester;
    use crate::workspace::WorkspaceBuilder;
    use futures::executor::block_on;

    #[test]
    fn test() {
        let mut builder = WorkspaceBuilder::new();
        let uri1 = builder.document("foo.tex", "\\label{foo}\n\\include{bar}");
        let uri2 = builder.document("bar.tex", "\\ref{foo}");
        let request = FeatureTester::new(builder.workspace, uri1.clone(), 0, 7, "bar").into();

        let changes = block_on(LatexLabelRenameProvider::execute(&request))
            .unwrap()
            .changes
            .unwrap();

        assert_eq!(2, changes.len());
        assert_eq!(
            vec![TextEdit::new(range::create(0, 7, 0, 10), "bar".to_owned()),],
            *changes.get(&uri1).unwrap()
        );
        assert_eq!(
            vec![TextEdit::new(range::create(0, 5, 0, 8), "bar".to_owned()),],
            *changes.get(&uri2).unwrap()
        );
    }

    #[test]
    fn test_command_args() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "\\foo{bar}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 5, "baz").into();

        let edit = block_on(LatexLabelRenameProvider::execute(&request));

        assert_eq!(None, edit);
    }

    #[test]
    fn test_bibtex() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.bib", "");
        let request = FeatureTester::new(builder.workspace, uri, 0, 0, "baz").into();

        let edit = block_on(LatexLabelRenameProvider::execute(&request));

        assert_eq!(None, edit);
    }
}
