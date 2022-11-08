use anyhow::Result;
use assert_unordered::assert_eq_unordered;
use lsp_types::{
    request::DocumentLinkRequest, ClientCapabilities, DocumentLink, DocumentLinkParams,
    TextDocumentIdentifier,
};

use crate::tests::{client::Client, fixture};

fn check(fixture: &str) -> Result<()> {
    let mut client = Client::spawn()?;
    client.initialize(ClientCapabilities::default(), None)?;

    let fixture = fixture::parse(fixture);
    for file in fixture.files {
        client.open(file.name, file.lang, file.text)?;
    }

    let mut expected_links = Vec::new();
    for ranges in fixture.ranges.values() {
        expected_links.push(DocumentLink {
            range: ranges[&1].range,
            target: Some(client.uri(ranges[&2].name)?),
            tooltip: None,
            data: None,
        });
    }

    let actual_links = client
        .request::<DocumentLinkRequest>(DocumentLinkParams {
            text_document: TextDocumentIdentifier::new(client.uri(fixture.cursor.unwrap().name)?),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        })?
        .unwrap_or_default();

    client.shutdown()?;

    assert_eq_unordered!(actual_links, expected_links);
    Ok(())
}

#[test]
fn document_include() -> Result<()> {
    check(
        r#"
%TEX foo.tex
%SRC \input{bar.tex}
%1.1        ^^^^^^^
%CUR ^

%TEX bar.tex
%SRC 
%1.2 
"#,
    )
}

#[test]
fn document_import() -> Result<()> {
    check(
        r#"
%TEX foo.tex
%SRC \import{.}{bar/baz}
%1.1            ^^^^^^^
%CUR ^

%TEX bar/baz.tex
%SRC 
%1.2 
"#,
    )
}
