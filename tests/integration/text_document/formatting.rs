use anyhow::Result;
use insta::assert_snapshot;
use lsp_types::{
    request::Formatting, ClientCapabilities, DocumentFormattingParams, FormattingOptions,
    TextDocumentIdentifier,
};
use texlab::{LineIndex, LineIndexExt};

use crate::{client::Client, fixture};

fn format(fixture: &str) -> Result<String> {
    let mut client = Client::spawn()?;
    client.initialize(ClientCapabilities::default(), None)?;

    let fixture = fixture::parse(fixture);
    let file = fixture.files.into_iter().next().unwrap();
    let old_text = file.text.clone();
    client.open(file.name, file.lang, file.text)?;

    let actual_edits = client
        .request::<Formatting>(DocumentFormattingParams {
            text_document: TextDocumentIdentifier::new(client.uri(file.name)?),
            work_done_progress_params: Default::default(),
            options: FormattingOptions {
                insert_spaces: true,
                tab_size: 4,
                ..Default::default()
            },
        })?
        .unwrap_or_default();

    client.shutdown()?;

    let line_index = LineIndex::new(&old_text);
    let mut actual_text = old_text;
    for edit in actual_edits.into_iter().rev() {
        let range = line_index.offset_lsp_range(edit.range);
        actual_text.replace_range::<std::ops::Range<usize>>(range.into(), &edit.new_text);
    }

    Ok(actual_text)
}

#[test]
fn bibtex_internal_wrap_long_lines() -> Result<()> {
    assert_snapshot!(format(
        r#"
%BIB main.bib
%SRC @article{foo, bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit.
%SRC Lorem ipsum dolor sit amet,
%SRC consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit.},}"#,
    )?);

    Ok(())
}

#[test]
fn bibtex_internal_multiple_entries() -> Result<()> {
    assert_snapshot!(format(
        r#"
%BIB main.bib
%SRC @article{foo, bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit amet, 
%SRC consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit.},}
%SRC 
%SRC @article{foo, bar = {Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit amet, 
%SRC consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit.},}""#,
    )?);

    Ok(())
}

#[test]
fn bibtex_internal_trailing_comma() -> Result<()> {
    assert_snapshot!(format(
        r#"
%BIB main.bib
%SRC @article{foo, bar = baz}"#,
    )?);

    Ok(())
}

#[test]
fn bibtex_internal_insert_braces() -> Result<()> {
    assert_snapshot!(format(
        r#"
%BIB main.bib
%SRC @article{foo, bar = baz,"#,
    )?);

    Ok(())
}

#[test]
fn bibtex_internal_command() -> Result<()> {
    assert_snapshot!(format(
        r#"
%BIB main.bib
%SRC @article{foo, bar = "\baz",}"#,
    )?);

    Ok(())
}

#[test]
fn bibtex_internal_join_strings() -> Result<()> {
    assert_snapshot!(format(
        r#"
%BIB main.bib
%SRC @article{foo, bar = "baz" # "qux"}"#,
    )?);

    Ok(())
}

#[test]
fn bibtex_internal_parens() -> Result<()> {
    assert_snapshot!(format(
        r#"
%BIB main.bib
%SRC @article(foo,)"#,
    )?);

    Ok(())
}

#[test]
fn bibtex_internal_string() -> Result<()> {
    assert_snapshot!(format(
        r#"
%BIB main.bib
%SRC @string{foo="bar"}"#,
    )?);

    Ok(())
}

#[test]
fn bibtex_internal_preamble() -> Result<()> {
    assert_snapshot!(format(
        r#"
%BIB main.bib
%SRC @preamble{
%SRC     "foo bar baz" }"#,
    )?);

    Ok(())
}
