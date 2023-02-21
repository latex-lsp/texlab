use assert_unordered::assert_eq_unordered;
use lsp_types::{
    request::DocumentHighlightRequest, ClientCapabilities, DocumentHighlight,
    DocumentHighlightKind, DocumentHighlightParams,
};

use crate::fixture::TestBed;

fn check(fixture: &str, highlight_kinds: &[DocumentHighlightKind]) {
    let test_bed = TestBed::new(fixture).unwrap();
    test_bed.initialize(ClientCapabilities::default()).unwrap();

    let expected: Vec<_> = test_bed
        .locations()
        .iter()
        .zip(highlight_kinds)
        .map(|(location, kind)| DocumentHighlight {
            range: location.range,
            kind: Some(*kind),
        })
        .collect();

    let text_document_position_params = test_bed.cursor().unwrap();
    let actual = test_bed
        .client()
        .send_request::<DocumentHighlightRequest>(DocumentHighlightParams {
            text_document_position_params,
            partial_result_params: Default::default(),
            work_done_progress_params: Default::default(),
        })
        .unwrap()
        .unwrap_or_default();

    assert_eq_unordered!(actual, expected);
}

#[test]
fn test_label() {
    check(
        r#"
%! main.tex
\label{foo}
        |
       ^^^
\ref{foo}
     ^^^
\label{bar}
"#,
        &[DocumentHighlightKind::WRITE, DocumentHighlightKind::READ],
    )
}
