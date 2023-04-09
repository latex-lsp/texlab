use base_db::LineIndex;
use insta::assert_snapshot;
use lsp_types::{
    request::Formatting, ClientCapabilities, DocumentFormattingParams, FormattingOptions,
};
use texlab::util::line_index_ext::LineIndexExt;

use crate::fixture::TestBed;

fn format(fixture: &str) -> String {
    let test_bed = TestBed::new(fixture).unwrap();
    test_bed.initialize(ClientCapabilities::default()).unwrap();

    let text_document = test_bed.cursor().unwrap().text_document;
    let edits = test_bed
        .client()
        .send_request::<Formatting>(DocumentFormattingParams {
            text_document,
            work_done_progress_params: Default::default(),
            options: FormattingOptions {
                insert_spaces: true,
                tab_size: 4,
                ..Default::default()
            },
        })
        .unwrap()
        .unwrap_or_default();

    let old_text = &test_bed.documents()[0].text;
    let line_index = LineIndex::new(old_text);
    let mut new_text = String::from(old_text);
    for edit in edits.into_iter().rev() {
        let range = line_index.offset_lsp_range(edit.range);
        new_text.replace_range::<std::ops::Range<usize>>(range.into(), &edit.new_text);
    }

    new_text
}

#[test]
fn bibtex_internal_wrap_long_lines() {
    assert_snapshot!(format(
        r#"
%! main.bib
@article{foo, bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit.
Lorem ipsum dolor sit amet,
consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit.},}
|"#,
    ));
}

#[test]
fn bibtex_internal_multiple_entries() {
    assert_snapshot!(format(
        r#"
%! main.bib
@article{foo, bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit amet, 
consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit.},}

@article{foo, bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit amet, 
consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit.},}"
|"#,
    ));
}

#[test]
fn bibtex_internal_trailing_comma() {
    assert_snapshot!(format(
        r#"
%! main.bib
@article{foo, bar = baz}
|"#,
    ));
}

#[test]
fn bibtex_internal_insert_braces() {
    assert_snapshot!(format(
        r#"
%! main.bib
@article{foo, bar = baz,
|"#,
    ));
}

#[test]
fn bibtex_internal_command() {
    assert_snapshot!(format(
        r#"
%! main.bib
@article{foo, bar = "\baz",}
|"#,
    ));
}

#[test]
fn bibtex_internal_join_strings() {
    assert_snapshot!(format(
        r#"
%! main.bib
@article{foo, bar = "baz" # "qux"}
|"#,
    ));
}

#[test]
fn bibtex_internal_parens() {
    assert_snapshot!(format(
        r#"
%! main.bib
@article(foo,)
|"#,
    ));
}

#[test]
fn bibtex_internal_string() {
    assert_snapshot!(format(
        r#"
%! main.bib
@string{foo="bar"}
|"#,
    ));
}

#[test]
fn bibtex_internal_preamble() {
    assert_snapshot!(format(
        r#"
%! main.bib
@preamble{
    "foo bar baz" }
|"#,
    ));
}
