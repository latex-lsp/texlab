use futures_boxed::boxed;
use std::collections::HashMap;
use texlab_protocol::RangeExt;
use texlab_protocol::*;
use texlab_syntax::*;
use texlab_workspace::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BibtexEntryPrepareRenameProvider;

impl FeatureProvider for BibtexEntryPrepareRenameProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Range>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<TextDocumentPositionParams>,
    ) -> Option<Range> {
        find_key(&request.document().tree, request.params.position).map(Span::range)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BibtexEntryRenameProvider;

impl FeatureProvider for BibtexEntryRenameProvider {
    type Params = RenameParams;
    type Output = Option<WorkspaceEdit>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<RenameParams>,
    ) -> Option<WorkspaceEdit> {
        let key_name = find_key(
            &request.document().tree,
            request.params.text_document_position.position,
        )?;
        let mut changes = HashMap::new();
        for document in request.related_documents() {
            let mut edits = Vec::new();
            match &document.tree {
                SyntaxTree::Latex(tree) => {
                    tree.citations
                        .iter()
                        .flat_map(LatexCitation::keys)
                        .filter(|citation| citation.text() == key_name.text)
                        .map(|citation| {
                            TextEdit::new(citation.range(), request.params.new_name.clone())
                        })
                        .for_each(|edit| edits.push(edit));
                }
                SyntaxTree::Bibtex(tree) => {
                    for entry in tree.entries() {
                        if let Some(key) = &entry.key {
                            if key.text() == key_name.text {
                                edits.push(TextEdit::new(
                                    key.range(),
                                    request.params.new_name.clone(),
                                ));
                            }
                        }
                    }
                }
            };
            changes.insert(document.uri.clone().into(), edits);
        }
        Some(WorkspaceEdit::new(changes))
    }
}

fn find_key(tree: &SyntaxTree, position: Position) -> Option<&Span> {
    match tree {
        SyntaxTree::Latex(tree) => {
            for citation in &tree.citations {
                let keys = citation.keys();
                for key in keys {
                    if key.range().contains(position) {
                        return Some(&key.span);
                    }
                }
            }
            None
        }
        SyntaxTree::Bibtex(tree) => {
            for entry in tree.entries() {
                if let Some(key) = &entry.key {
                    if key.range().contains(position) {
                        return Some(&key.span);
                    }
                }
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use texlab_protocol::Position;

    #[test]
    fn entry() {
        let edit = test_feature(
            BibtexEntryRenameProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.bib", "@article{foo, bar = baz}"),
                    FeatureSpec::file("bar.tex", "\\addbibresource{foo.bib}\n\\cite{foo}"),
                ],
                main_file: "foo.bib",
                position: Position::new(0, 9),
                new_name: "qux",
                ..FeatureSpec::default()
            },
        );
        let mut changes = HashMap::new();
        changes.insert(
            FeatureSpec::uri("foo.bib"),
            vec![TextEdit::new(Range::new_simple(0, 9, 0, 12), "qux".into())],
        );
        changes.insert(
            FeatureSpec::uri("bar.tex"),
            vec![TextEdit::new(Range::new_simple(1, 6, 1, 9), "qux".into())],
        );
        assert_eq!(edit, Some(WorkspaceEdit::new(changes)));
    }

    #[test]
    fn citation() {
        let edit = test_feature(
            BibtexEntryRenameProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.bib", "@article{foo, bar = baz}"),
                    FeatureSpec::file("bar.tex", "\\addbibresource{foo.bib}\n\\cite{foo}"),
                ],
                main_file: "bar.tex",
                position: Position::new(1, 6),
                new_name: "qux",
                ..FeatureSpec::default()
            },
        );
        let mut changes = HashMap::new();
        changes.insert(
            FeatureSpec::uri("foo.bib"),
            vec![TextEdit::new(Range::new_simple(0, 9, 0, 12), "qux".into())],
        );
        changes.insert(
            FeatureSpec::uri("bar.tex"),
            vec![TextEdit::new(Range::new_simple(1, 6, 1, 9), "qux".into())],
        );
        assert_eq!(edit, Some(WorkspaceEdit::new(changes)));
    }

    #[test]
    fn field_name() {
        let edit = test_feature(
            BibtexEntryRenameProvider,
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
