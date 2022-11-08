use anyhow::Result;
use assert_unordered::assert_eq_unordered;
use lsp_types::{
    request::DocumentHighlightRequest, ClientCapabilities, DocumentHighlight,
    DocumentHighlightKind, DocumentHighlightParams,
};

use crate::tests::{client::Client, fixture};

fn check(fixture: &str) -> Result<()> {
    let mut client = Client::spawn()?;
    client.initialize(ClientCapabilities::default(), None)?;

    let fixture = fixture::parse(fixture);
    for file in fixture.files {
        client.open(file.name, file.lang, file.text)?;
    }

    let mut expected_highlights = Vec::new();
    for ranges in fixture.ranges.values() {
        let (i, file_range) = ranges.iter().next().unwrap();
        let kind = match i {
            1 => DocumentHighlightKind::TEXT,
            2 => DocumentHighlightKind::READ,
            3 => DocumentHighlightKind::WRITE,
            _ => unreachable!(),
        };

        expected_highlights.push(DocumentHighlight {
            range: file_range.range,
            kind: Some(kind),
        });
    }

    let actual_highlights = client
        .request::<DocumentHighlightRequest>(DocumentHighlightParams {
            text_document_position_params: fixture.cursor.unwrap().into_params(&client)?,
            partial_result_params: Default::default(),
            work_done_progress_params: Default::default(),
        })?
        .unwrap_or_default();

    client.shutdown()?;

    assert_eq_unordered!(actual_highlights, expected_highlights);
    Ok(())
}

#[test]
fn test_label() -> Result<()> {
    check(
        r#"
%TEX main.tex
%SRC \label{foo}
%CUR         ^
%1.3        ^^^
%SRC \ref{foo}
%2.2      ^^^
%SRC \label{bar}
"#,
    )
}
