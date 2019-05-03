use crate::feature::FeatureRequest;
use crate::range;
use crate::syntax::bibtex::ast::BibtexDeclaration;
use crate::syntax::bibtex::BibtexSyntaxTree;
use crate::syntax::latex::{LatexCitationAnalyzer, LatexSyntaxTree, LatexVisitor, SyntaxNode};
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
    use crate::completion::latex::data::types::LatexComponentDatabase;
    use crate::feature::FeatureSpec;
    use crate::range;
    use crate::test_feature;
    use lsp_types::Position;

    #[test]
    fn test_entry() {
        let edit = test_feature!(
            BibtexEntryRenameProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.bib", "@article{foo, bar = baz}"),
                    FeatureSpec::file("bar.tex", "\\addbibresource{foo.bib}\n\\cite{foo}"),
                ],
                main_file: "foo.bib",
                position: Position::new(0, 9),
                new_name: "qux",
                component_database: LatexComponentDatabase::default(),
            }
        );
        let mut changes = HashMap::new();
        changes.insert(
            FeatureSpec::uri("foo.bib"),
            vec![TextEdit::new(range::create(0, 9, 0, 12), "qux".to_owned())],
        );
        changes.insert(
            FeatureSpec::uri("bar.tex"),
            vec![TextEdit::new(range::create(1, 6, 1, 9), "qux".to_owned())],
        );
        assert_eq!(edit, Some(WorkspaceEdit::new(changes)));
    }

    #[test]
    fn test_citation() {
        let edit = test_feature!(
            BibtexEntryRenameProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.bib", "@article{foo, bar = baz}"),
                    FeatureSpec::file("bar.tex", "\\addbibresource{foo.bib}\n\\cite{foo}"),
                ],
                main_file: "bar.tex",
                position: Position::new(1, 6),
                new_name: "qux",
                component_database: LatexComponentDatabase::default(),
            }
        );
        let mut changes = HashMap::new();
        changes.insert(
            FeatureSpec::uri("foo.bib"),
            vec![TextEdit::new(range::create(0, 9, 0, 12), "qux".to_owned())],
        );
        changes.insert(
            FeatureSpec::uri("bar.tex"),
            vec![TextEdit::new(range::create(1, 6, 1, 9), "qux".to_owned())],
        );
        assert_eq!(edit, Some(WorkspaceEdit::new(changes)));
    }

    #[test]
    fn test_field_name() {
        let edit = test_feature!(
            BibtexEntryRenameProvider,
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
