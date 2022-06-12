use anyhow::Result;
use assert_unordered::assert_eq_unordered;
use lsp_types::{
    request::GotoDefinition, ClientCapabilities, GotoDefinitionParams, GotoDefinitionResponse,
    LocationLink,
};

use crate::lsp::{client::Client, fixture};

fn check(fixture: &str) -> Result<()> {
    let mut client = Client::spawn()?;
    client.initialize(ClientCapabilities::default(), None)?;

    let fixture = fixture::parse(fixture);
    for file in fixture.files {
        client.open(file.name, file.lang, file.text)?;
    }

    let mut expected_links = Vec::new();
    for ranges in fixture.ranges.values() {
        expected_links.push(LocationLink {
            origin_selection_range: Some(ranges[&1].range),
            target_uri: client.uri(ranges[&2].name)?,
            target_range: ranges[&2].range,
            target_selection_range: ranges[&3].range,
        });
    }

    let actual_links = client
        .request::<GotoDefinition>(GotoDefinitionParams {
            text_document_position_params: fixture.cursor.unwrap().into_params(&client)?,
            partial_result_params: Default::default(),
            work_done_progress_params: Default::default(),
        })?
        .map_or(Vec::new(), |actual| match actual {
            GotoDefinitionResponse::Link(links) => links,
            GotoDefinitionResponse::Array(_) | GotoDefinitionResponse::Scalar(_) => unreachable!(),
        });

    client.shutdown()?;

    assert_eq_unordered!(actual_links, expected_links);
    Ok(())
}

#[test]
fn command_definition() -> Result<()> {
    check(
        r#"
%TEX main.tex
%SRC \DeclareMathOperator{\foo}{foo}
%1.3                      ^^^^
%1.2 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
%SRC \foo
%CUR   ^
%1.1 ^^^^
"#,
    )
}

#[test]
fn document() -> Result<()> {
    check(
        r#"
%TEX foo.tex
%SRC \addbibresource{baz.bib}
%CUR                   ^
%1.1                 ^^^^^^^

%TEX bar.bib
%SRC @article{foo, bar = {baz}}

%TEX baz.bib
%SRC @article{foo, bar = {baz}}
%1.3 
%1.2 
"#,
    )
}

#[test]
fn entry() -> Result<()> {
    check(
        r#"
%TEX foo.tex
%SRC \addbibresource{baz.bib}
%SRC \cite{foo}
%CUR       ^
%1.1       ^^^

%BIB bar.bib
%SRC @article{foo, bar = {baz}}

%BIB baz.bib
%SRC @article{foo, bar = {baz}}
%1.3          ^^^
%1.2 ^^^^^^^^^^^^^^^^^^^^^^^^^^
"#,
    )
}

#[test]
fn string_simple() -> Result<()> {
    check(
        r#"
%BIB main.bib
%SRC @string{foo = {bar}}
%1.3         ^^^
%1.2 ^^^^^^^^^^^^^^^^^^^^
%SRC @article{bar, author = foo}
%CUR                         ^
%1.1                        ^^^
"#,
    )
}

#[test]
fn string_join() -> Result<()> {
    check(
        r#"
%BIB main.bib
%SRC @string{foo = {bar}}
%1.3         ^^^
%1.2 ^^^^^^^^^^^^^^^^^^^^
%SRC @article{bar, author = foo # "bar"}
%CUR                         ^
%1.1                        ^^^
"#,
    )
}

#[test]
fn string_field() -> Result<()> {
    check(
        r#"
%BIB main.bib
%SRC @string{foo = {bar}}
%SRC @article{bar, author = foo # "bar"}
%CUR                 ^
"#,
    )
}
