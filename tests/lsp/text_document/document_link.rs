use std::path::PathBuf;

use insta::assert_json_snapshot;
use lsp_types::{
    request::DocumentLinkRequest, ClientCapabilities, DocumentLink, DocumentLinkParams, Url,
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

    let root = PathBuf::from("/");
    for link in &mut links {
        let path = link.target.take().unwrap().to_file_path().unwrap();
        let path = path.strip_prefix(test_bed.directory()).unwrap_or(&path);
        let path = root.join(path);
        link.target = Some(Url::from_file_path(path).unwrap());
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
