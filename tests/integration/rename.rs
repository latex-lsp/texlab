use anyhow::Result;
use insta::assert_json_snapshot;
use lsp_types::{notification::DidOpenTextDocument, request::Rename};
use serde_json::json;

use crate::{redaction::redact_uri, server::ServerWrapper};

macro_rules! verify {
    ($server:expr, $result:expr) => {
        let state = $server.shutdown()?;

        let path = state.directory.path().to_owned();
        assert_json_snapshot!(
            $result,
            {
                ".changes.$key" => insta::dynamic_redaction(redact_uri(path)),
                ".changes" => insta::sorted_redaction(),
            }
        );
    };
}

#[test]
fn test_command() -> Result<()> {
    let mut server = ServerWrapper::spawn()?;
    server.initialize(json!({ "capabilities": {} }))?;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("foo.tex"),
            "languageId": "latex",
            "version": 1i32,
            "text": r#"\baz\include{bar.tex}"#
        }
    }))?;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("bar.tex"),
            "languageId": "latex",
            "version": 1i32,
            "text": r#"\baz"#
        }
    }))?;

    let result = server.request::<Rename>(json!({
        "textDocument": {
            "uri": server.uri("foo.tex")
        },
        "position": {
            "line": 0u32,
            "character": 2u32
        },
        "newName": "qux"
    }))?;

    verify!(server, result);
    Ok(())
}

#[test]
fn test_entry() -> Result<()> {
    let mut server = ServerWrapper::spawn()?;
    server.initialize(json!({ "capabilities": {} }))?;

    let text1 = r#"@article{foo, bar = baz}"#;
    let text2 = r#"\addbibresource{main.bib}
\cite{foo}"#;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("main.bib"),
            "languageId": "bibtex",
            "version": 1i32,
            "text": text1
        }
    }))?;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("main.tex"),
            "languageId": "latex",
            "version": 1i32,
            "text": text2
        }
    }))?;

    let result = server.request::<Rename>(json!({
        "textDocument": {
            "uri": server.uri("main.bib")
        },
        "position": {
            "line": 0u32,
            "character": 9u32
        },
        "newName": "qux"
    }))?;

    verify!(server, result);
    Ok(())
}

#[test]
fn test_citation() -> Result<()> {
    let mut server = ServerWrapper::spawn()?;
    server.initialize(json!({ "capabilities": {} }))?;

    let text1 = r#"@article{foo, bar = baz}"#;
    let text2 = r#"\addbibresource{main.bib}
\cite{foo}"#;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("main.bib"),
            "languageId": "bibtex",
            "version": 1i32,
            "text": text1
        }
    }))?;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("main.tex"),
            "languageId": "latex",
            "version": 1i32,
            "text": text2
        }
    }))?;

    let result = server.request::<Rename>(json!({
        "textDocument": {
            "uri": server.uri("main.tex")
        },
        "position": {
            "line": 1u32,
            "character": 6u32
        },
        "newName": "qux"
    }))?;

    verify!(server, result);
    Ok(())
}

#[test]
fn test_label() -> Result<()> {
    let mut server = ServerWrapper::spawn()?;
    server.initialize(json!({ "capabilities": {} }))?;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("foo.tex"),
            "languageId": "latex",
            "version": 1i32,
            "text": r#"\label{foo}\include{bar}"#
        }
    }))?;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("bar.tex"),
            "languageId": "latex",
            "version": 1i32,
            "text": r#"\ref{foo}"#
        }
    }))?;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("baz.tex"),
            "languageId": "latex",
            "version": 1i32,
            "text": r#"\ref{foo}"#
        }
    }))?;

    let result = server.request::<Rename>(json!({
        "textDocument": {
            "uri": server.uri("foo.tex")
        },
        "position": {
            "line": 0u32,
            "character": 7u32
        },
        "newName": "bar"
    }))?;

    verify!(server, result);
    Ok(())
}
