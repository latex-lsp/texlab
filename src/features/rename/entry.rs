use std::collections::HashMap;

use cancellation::CancellationToken;
use lsp_types::{Range, RenameParams, TextEdit, WorkspaceEdit};

use crate::{
    features::cursor::{Cursor, CursorContext, HasPosition},
    syntax::{bibtex, latex, CstNode},
    DocumentData, LineIndexExt,
};

pub fn prepare_entry_rename<P: HasPosition>(
    context: &CursorContext<P>,
    _cancellation_token: &CancellationToken,
) -> Option<Range> {
    let range = match &context.cursor {
        Cursor::Latex(token) if token.kind() == latex::WORD => {
            let group = latex::CurlyGroupWordList::cast(token.parent())?;
            latex::Citation::cast(group.syntax().parent()?)?;
            token.text_range()
        }
        Cursor::Bibtex(token) if token.kind() == bibtex::WORD => {
            bibtex::Entry::cast(token.parent())?;
            token.text_range()
        }
        _ => return None,
    };

    Some(
        context
            .request
            .main_document()
            .line_index
            .line_col_lsp_range(range),
    )
}

pub fn rename_entry(
    context: &CursorContext<RenameParams>,
    cancellation_token: &CancellationToken,
) -> Option<WorkspaceEdit> {
    cancellation_token.result().ok()?;
    prepare_entry_rename(context, cancellation_token)?;
    let key_text = context
        .cursor
        .as_latex()
        .map(|token| token.text())
        .or_else(|| context.cursor.as_bibtex().map(|token| token.text()))?;

    let mut changes = HashMap::new();
    for document in &context.request.subset.documents {
        cancellation_token.result().ok()?;
        match &document.data {
            DocumentData::Latex(data) => {
                let edits: Vec<_> = data
                    .root
                    .descendants()
                    .filter_map(latex::Citation::cast)
                    .filter_map(|citation| citation.key_list())
                    .flat_map(|keys| keys.words())
                    .filter(|key| key.text() == key_text)
                    .map(|key| document.line_index.line_col_lsp_range(key.text_range()))
                    .map(|range| TextEdit::new(range, context.request.params.new_name.clone()))
                    .collect();
                changes.insert(document.uri.as_ref().clone().into(), edits);
            }
            DocumentData::Bibtex(data) => {
                let edits: Vec<_> = data
                    .root
                    .descendants()
                    .filter_map(bibtex::Entry::cast)
                    .filter_map(|entry| entry.key())
                    .filter(|key| key.text() == key_text)
                    .map(|key| document.line_index.line_col_lsp_range(key.text_range()))
                    .map(|range| TextEdit::new(range, context.request.params.new_name.clone()))
                    .collect();
                changes.insert(document.uri.as_ref().clone().into(), edits);
            }
            DocumentData::BuildLog(_) => {}
        }
    }

    Some(WorkspaceEdit::new(changes))
}

#[cfg(test)]
mod tests {
    use lsp_types::TextEdit;

    use crate::{features::testing::FeatureTester, RangeExt};

    use super::*;

    #[test]
    fn test_entry() {
        let tester = FeatureTester::builder()
            .files(vec![
                ("main.bib", r#"@article{foo, bar = baz}"#),
                ("main.tex", "\\addbibresource{main.bib}\n\\cite{foo}"),
            ])
            .main("main.bib")
            .line(0)
            .character(9)
            .new_name("qux")
            .build();

        let uri1 = tester.uri("main.bib");
        let uri2 = tester.uri("main.tex");
        let request = tester.rename();

        let context = CursorContext::new(request);
        let actual_edit = rename_entry(&context, CancellationToken::none()).unwrap();

        let mut expected_changes = HashMap::new();
        expected_changes.insert(
            uri1.as_ref().clone().into(),
            vec![TextEdit::new(Range::new_simple(0, 9, 0, 12), "qux".into())],
        );
        expected_changes.insert(
            uri2.as_ref().clone().into(),
            vec![TextEdit::new(Range::new_simple(1, 6, 1, 9), "qux".into())],
        );
        let expected_edit = WorkspaceEdit::new(expected_changes);

        assert_eq!(actual_edit, expected_edit);
    }

    #[test]
    fn test_citation() {
        let tester = FeatureTester::builder()
            .files(vec![
                ("main.bib", r#"@article{foo, bar = baz}"#),
                ("main.tex", "\\addbibresource{main.bib}\n\\cite{foo}"),
            ])
            .main("main.tex")
            .line(1)
            .character(6)
            .new_name("qux")
            .build();

        let uri1 = tester.uri("main.bib");
        let uri2 = tester.uri("main.tex");
        let request = tester.rename();

        let context = CursorContext::new(request);
        let actual_edit = rename_entry(&context, CancellationToken::none()).unwrap();

        let mut expected_changes = HashMap::new();
        expected_changes.insert(
            uri1.as_ref().clone().into(),
            vec![TextEdit::new(Range::new_simple(0, 9, 0, 12), "qux".into())],
        );
        expected_changes.insert(
            uri2.as_ref().clone().into(),
            vec![TextEdit::new(Range::new_simple(1, 6, 1, 9), "qux".into())],
        );
        let expected_edit = WorkspaceEdit::new(expected_changes);

        assert_eq!(actual_edit, expected_edit);
    }
}
