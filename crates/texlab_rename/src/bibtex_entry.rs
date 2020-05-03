use async_trait::async_trait;
use std::collections::HashMap;
use texlab_feature::{DocumentContent, FeatureProvider, FeatureRequest};
use texlab_protocol::{
    Position, Range, RangeExt, RenameParams, TextDocumentPositionParams, TextEdit, WorkspaceEdit,
};
use texlab_syntax::{Span, SyntaxNode};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct BibtexEntryPrepareRenameProvider;

#[async_trait]
impl FeatureProvider for BibtexEntryPrepareRenameProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Range>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        find_key(&req.current().content, req.params.position).map(Span::range)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct BibtexEntryRenameProvider;

#[async_trait]
impl FeatureProvider for BibtexEntryRenameProvider {
    type Params = RenameParams;
    type Output = Option<WorkspaceEdit>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let key_name = find_key(
            &req.current().content,
            req.params.text_document_position.position,
        )?;
        let mut changes = HashMap::new();
        for doc in req.related() {
            let edits = match &doc.content {
                DocumentContent::Latex(table) => table
                    .citations
                    .iter()
                    .flat_map(|citation| citation.keys(&table))
                    .filter(|citation| citation.text() == key_name.text)
                    .map(|citation| TextEdit::new(citation.range(), req.params.new_name.clone()))
                    .collect(),
                DocumentContent::Bibtex(tree) => tree
                    .children(tree.root)
                    .filter_map(|node| tree.as_entry(node))
                    .filter_map(|entry| entry.key.as_ref())
                    .filter(|entry_key| entry_key.text() == key_name.text)
                    .map(|entry_key| TextEdit::new(entry_key.range(), req.params.new_name.clone()))
                    .collect(),
            };
            changes.insert(doc.uri.clone().into(), edits);
        }
        Some(WorkspaceEdit::new(changes))
    }
}

fn find_key(content: &DocumentContent, pos: Position) -> Option<&Span> {
    match content {
        DocumentContent::Latex(table) => table
            .citations
            .iter()
            .flat_map(|citation| citation.keys(&table))
            .find(|key| key.range().contains(pos))
            .map(|key| &key.span),
        DocumentContent::Bibtex(tree) => tree
            .children(tree.root)
            .filter_map(|node| tree.as_entry(node))
            .filter_map(|entry| entry.key.as_ref())
            .find(|key| key.range().contains(pos))
            .map(|key| &key.span),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use texlab_feature::FeatureTester;
    use texlab_protocol::RangeExt;

    #[tokio::test]
    async fn entry() {
        let actual_edit = FeatureTester::new()
            .file("main.bib", r#"@article{foo, bar = baz}"#)
            .file(
                "main.tex",
                indoc!(
                    r#"
                        \addbibresource{main.bib}
                        \cite{foo}
                    "#
                ),
            )
            .main("main.bib")
            .position(0, 9)
            .new_name("qux")
            .test_rename(BibtexEntryRenameProvider)
            .await
            .unwrap();

        let mut expected_changes = HashMap::new();
        expected_changes.insert(
            FeatureTester::uri("main.bib").into(),
            vec![TextEdit::new(Range::new_simple(0, 9, 0, 12), "qux".into())],
        );
        expected_changes.insert(
            FeatureTester::uri("main.tex").into(),
            vec![TextEdit::new(Range::new_simple(1, 6, 1, 9), "qux".into())],
        );
        let expected_edit = WorkspaceEdit::new(expected_changes);

        assert_eq!(actual_edit, expected_edit);
    }

    #[tokio::test]
    async fn citation() {
        let actual_edit = FeatureTester::new()
            .file("main.bib", r#"@article{foo, bar = baz}"#)
            .file(
                "main.tex",
                indoc!(
                    r#"
                    \addbibresource{main.bib}
                    \cite{foo}
                "#
                ),
            )
            .main("main.tex")
            .position(1, 6)
            .new_name("qux")
            .test_rename(BibtexEntryRenameProvider)
            .await
            .unwrap();

        let mut expected_changes = HashMap::new();
        expected_changes.insert(
            FeatureTester::uri("main.bib").into(),
            vec![TextEdit::new(Range::new_simple(0, 9, 0, 12), "qux".into())],
        );
        expected_changes.insert(
            FeatureTester::uri("main.tex").into(),
            vec![TextEdit::new(Range::new_simple(1, 6, 1, 9), "qux".into())],
        );
        let expected_edit = WorkspaceEdit::new(expected_changes);

        assert_eq!(actual_edit, expected_edit);
    }

    #[tokio::test]
    async fn field_name() {
        let actual_edit = FeatureTester::new()
            .file("main.bib", r#"@article{foo, bar = baz}"#)
            .main("main.bib")
            .position(0, 14)
            .new_name("qux")
            .test_rename(BibtexEntryRenameProvider)
            .await;

        assert_eq!(actual_edit, None);
    }

    #[tokio::test]
    async fn empty_latex_document() {
        let actual_edit = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .new_name("")
            .test_rename(BibtexEntryRenameProvider)
            .await;

        assert_eq!(actual_edit, None);
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_edit = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .new_name("")
            .test_rename(BibtexEntryRenameProvider)
            .await;

        assert_eq!(actual_edit, None);
    }
}
