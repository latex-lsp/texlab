use insta::assert_json_snapshot;
use lsp_types::{
    request::DocumentLinkRequest, ClientCapabilities, DocumentLink, DocumentLinkParams,
};

use crate::fixture::TestBed;

fn find_links(fixture: &str) -> Vec<DocumentLink> {
    let test_bed = TestBed::new(fixture).unwrap();
    test_bed.initialize(ClientCapabilities::default()).unwrap();

    let text_document_position = test_bed.cursor().unwrap();
    let mut links = test_bed
        .client()
        .send_request::<DocumentLinkRequest>(DocumentLinkParams {
            text_document: text_document_position.text_document,
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        })
        .unwrap()
        .unwrap_or_default();

    for link in &mut links {
        link.target = Some(test_bed.redact(link.target.as_ref().unwrap()));
    }

    links
}

#[test]
fn document_include() {
    assert_json_snapshot!(find_links(
        r#"
%! foo.tex
\input{bar.tex}
|

%! bar.tex"#,
    ))
}

#[test]
fn document_import() {
    assert_json_snapshot!(find_links(
        r#"
%! foo.tex
\import{.}{bar/baz}
|

%! bar/baz.tex"#,
    ))
}
