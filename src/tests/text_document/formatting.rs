use insta::assert_snapshot;
use lsp_types::{
    request::Formatting, ClientCapabilities, DocumentFormattingParams, FormattingOptions,
    TextDocumentIdentifier,
};

use crate::{
    tests::{client::Client, fixture},
    util::{line_index::LineIndex, line_index_ext::LineIndexExt},
};

fn format(fixture: &str) -> String {
    let mut client = Client::spawn();
    client.initialize(ClientCapabilities::default(), None);

    let fixture = fixture::parse(fixture);
    let file = fixture.files.into_iter().next().unwrap();
    let old_text = file.text.clone();
    client.open(file.name, file.lang, file.text);

    let actual_edits = client
        .request::<Formatting>(DocumentFormattingParams {
            text_document: TextDocumentIdentifier::new(client.uri(file.name)),
            work_done_progress_params: Default::default(),
            options: FormattingOptions {
                insert_spaces: true,
                tab_size: 4,
                ..Default::default()
            },
        })
        .unwrap()
        .unwrap_or_default();

    client.shutdown();

    let line_index = LineIndex::new(&old_text);
    let mut actual_text = old_text;
    for edit in actual_edits.into_iter().rev() {
        let range = line_index.offset_lsp_range(edit.range);
        actual_text.replace_range::<std::ops::Range<usize>>(range.into(), &edit.new_text);
    }

    actual_text
}

#[test]
fn bibtex_internal_wrap_long_lines() {
    assert_snapshot!(format(
        r#"
%BIB main.bib
%SRC @article{foo, bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit.
%SRC Lorem ipsum dolor sit amet,
%SRC consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit.},}"#,
    ));
}

#[test]
fn bibtex_internal_multiple_entries() {
    assert_snapshot!(format(
        r#"
%BIB main.bib
%SRC @article{foo, bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit amet, 
%SRC consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit.},}
%SRC 
%SRC @article{foo, bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit amet, 
%SRC consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit.},}""#,
    ));
}

#[test]
fn bibtex_internal_trailing_comma() {
    assert_snapshot!(format(
        r#"
%BIB main.bib
%SRC @article{foo, bar = baz}"#,
    ));
}

#[test]
fn bibtex_internal_insert_braces() {
    assert_snapshot!(format(
        r#"
%BIB main.bib
%SRC @article{foo, bar = baz,"#,
    ));
}

#[test]
fn bibtex_internal_command() {
    assert_snapshot!(format(
        r#"
%BIB main.bib
%SRC @article{foo, bar = "\baz",}"#,
    ));
}

#[test]
fn bibtex_internal_join_strings() {
    assert_snapshot!(format(
        r#"
%BIB main.bib
%SRC @article{foo, bar = "baz" # "qux"}"#,
    ));
}

#[test]
fn bibtex_internal_parens() {
    assert_snapshot!(format(
        r#"
%BIB main.bib
%SRC @article(foo,)"#,
    ));
}

#[test]
fn bibtex_internal_string() {
    assert_snapshot!(format(
        r#"
%BIB main.bib
%SRC @string{foo="bar"}"#,
    ));
}

#[test]
fn bibtex_internal_preamble() {
    assert_snapshot!(format(
        r#"
%BIB main.bib
%SRC @preamble{
%SRC     "foo bar baz" }"#,
    ));
}
