use crate::feature::FeatureRequest;
use crate::syntax::bibtex::BibtexSyntaxTree;
use crate::syntax::latex::LatexSyntaxTree;
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use lsp_types::*;
use std::borrow::Cow;
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
                    tree.citations
                        .iter()
                        .filter(|citation| citation.key().text() == key_name)
                        .map(|citation| {
                            TextEdit::new(
                                citation.key().range(),
                                Cow::from(request.params.new_name.clone()),
                            )
                        })
                        .for_each(|edit| edits.push(edit));
                }
                SyntaxTree::Bibtex(tree) => {
                    for entry in tree.entries() {
                        if let Some(key) = &entry.key {
                            if key.text() == key_name {
                                edits.push(TextEdit::new(
                                    key.range(),
                                    Cow::from(request.params.new_name.clone()),
                                ));
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
        for citation in &tree.citations {
            let key = citation.key();
            if key.range().contains(position) {
                return Some(key.text());
            }
        }
        None
    }

    fn find_entry(tree: &BibtexSyntaxTree, position: Position) -> Option<&str> {
        for entry in tree.entries() {
            if let Some(key) = &entry.key {
                if key.range().contains(position) {
                    return Some(&key.text());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::completion::LatexComponentDatabase;
    use crate::feature::FeatureSpec;
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
            vec![TextEdit::new(
                Range::new_simple(0, 9, 0, 12),
                Cow::from("qux"),
            )],
        );
        changes.insert(
            FeatureSpec::uri("bar.tex"),
            vec![TextEdit::new(
                Range::new_simple(1, 6, 1, 9),
                Cow::from("qux"),
            )],
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
            vec![TextEdit::new(
                Range::new_simple(0, 9, 0, 12),
                Cow::from("qux"),
            )],
        );
        changes.insert(
            FeatureSpec::uri("bar.tex"),
            vec![TextEdit::new(
                Range::new_simple(1, 6, 1, 9),
                Cow::from("qux"),
            )],
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
