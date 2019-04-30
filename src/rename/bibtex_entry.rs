use crate::feature::FeatureRequest;
use crate::range;
use crate::syntax::bibtex::ast::BibtexDeclaration;
use crate::syntax::bibtex::BibtexSyntaxTree;
use crate::syntax::latex::analysis::citation::LatexCitationAnalyzer;
use crate::syntax::latex::ast::LatexVisitor;
use crate::syntax::latex::LatexSyntaxTree;
use crate::syntax::text::Span;
use crate::syntax::text::SyntaxNode;
use crate::workspace::SyntaxTree;
use lsp_types::RenameParams;
use lsp_types::WorkspaceEdit;
use lsp_types::{Position, TextEdit};
use std::collections::HashMap;

pub struct BibtexEntryRenameProvider;

impl BibtexEntryRenameProvider {
    pub async fn execute(request: &FeatureRequest<RenameParams>) -> Option<WorkspaceEdit> {
        let key_name = match &request.document.tree {
            SyntaxTree::Latex(tree) => Self::find_citation(&tree, request.params.position),
            SyntaxTree::Bibtex(tree) => Self::find_entry(&tree, request.params.position),
        }?;

        let mut changes = HashMap::new();
        for document in &request.related_documents {
            let mut edits = Vec::new();
            match &document.tree {
                SyntaxTree::Latex(tree) => {
                    let mut analyzer = LatexCitationAnalyzer::new();
                    analyzer.visit_root(&tree.root);
                    analyzer
                        .citations
                        .iter()
                        .filter(|citation| citation.key.text() == key_name)
                        .map(|citation| {
                            TextEdit::new(citation.key.range(), request.params.new_name.clone())
                        })
                        .for_each(|edit| edits.push(edit));
                }
                SyntaxTree::Bibtex(tree) => {
                    for declaration in &tree.root.children {
                        if let BibtexDeclaration::Entry(entry) = declaration {
                            if let Some(key) = &entry.key {
                                if key.text() == key_name {
                                    edits.push(TextEdit::new(
                                        key.range(),
                                        request.params.new_name.clone(),
                                    ));
                                }
                            }
                        }
                    }
                }
            };
            changes.insert(document.uri.clone(), edits);
        }
        Some(WorkspaceEdit::new(changes))
    }

    fn find_citation(tree: &LatexSyntaxTree, position: Position) -> Option<&str> {
        let mut analyzer = LatexCitationAnalyzer::new();
        analyzer.visit_root(&tree.root);
        for citation in &analyzer.citations {
            if range::contains(citation.key.range(), position) {
                return Some(&citation.key.text());
            }
        }
        None
    }

    fn find_entry(tree: &BibtexSyntaxTree, position: Position) -> Option<&str> {
        for declaration in &tree.root.children {
            if let BibtexDeclaration::Entry(entry) = declaration {
                if let Some(key) = &entry.key {
                    if range::contains(key.range(), position) {
                        return Some(&key.text());
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
    use crate::range;
    use crate::workspace::WorkspaceBuilder;
    use futures::executor::block_on;

    #[test]
    fn test() {
        struct Row {
            index: usize,
            line: u64,
            character: u64,
        };

        for row in vec![
            Row {
                index: 0,
                line: 0,
                character: 9,
            },
            Row {
                index: 1,
                line: 1,
                character: 6,
            },
        ] {
            let mut builder = WorkspaceBuilder::new();
            let uri1 = builder.document("foo.bib", "@article{foo, bar = baz}");
            let uri2 = builder.document("bar.tex", "\\addbibresource{foo.bib}\n\\cite{foo}");
            let uri = vec![&uri1, &uri2][row.index].clone();
            let request =
                FeatureTester::new(builder.workspace, uri, row.line, row.character, "qux").into();

            let changes = block_on(BibtexEntryRenameProvider::execute(&request))
                .unwrap()
                .changes
                .unwrap();

            assert_eq!(2, changes.len());
            assert_eq!(
                vec![TextEdit::new(range::create(0, 9, 0, 12), "qux".to_owned())],
                *changes.get(&uri1).unwrap()
            );
            assert_eq!(
                vec![TextEdit::new(range::create(1, 6, 1, 9), "qux".to_owned())],
                *changes.get(&uri2).unwrap()
            );
        }
    }

    #[test]
    fn test_field_name() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.bib", "@article{foo, bar = baz}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 14, "qux").into();

        let edit = block_on(BibtexEntryRenameProvider::execute(&request));

        assert_eq!(None, edit);
    }
}
