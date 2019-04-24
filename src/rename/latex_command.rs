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
    use crate::feature::FeatureTester;
    use crate::workspace::WorkspaceBuilder;
    use futures::executor;

    #[test]
    fn test() {
        let mut builder = WorkspaceBuilder::new();
        let uri1 = builder.document("foo.tex", "\\include{bar.tex}\n\\baz");
        let uri2 = builder.document("bar.tex", "\\baz");
        let request = FeatureTester::new(builder.workspace, uri1.clone(), 1, 2, "qux").into();

        let changes = executor::block_on(LatexCommandRenameProvider::execute(&request))
            .unwrap()
            .changes
            .unwrap();

        assert_eq!(2, changes.len());
        assert_eq!(
            vec![TextEdit::new(range::create(1, 0, 1, 4), "\\qux".to_owned())],
            *changes.get(&uri1).unwrap()
        );
        assert_eq!(
            vec![TextEdit::new(range::create(0, 0, 0, 4), "\\qux".to_owned())],
            *changes.get(&uri2).unwrap()
        );
    }

    #[test]
    fn test_bibtex() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.bib", "\\foo");
        let request = FeatureTester::new(builder.workspace, uri, 0, 1, "baz").into();

        let edit = executor::block_on(LatexCommandRenameProvider::execute(&request));

        assert_eq!(None, edit);
    }
}
