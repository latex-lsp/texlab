use anyhow::Result;
use lsp_types::{notification::DidOpenTextDocument, request::References};
use serde_json::json;

use crate::{redaction::redact_uri, server::ServerWrapper};

macro_rules! verify {
    ($server:expr, $result:expr) => {
        let state = $server.shutdown()?;

        let path = state.directory.path().to_owned();
        insta::assert_json_snapshot!(
            $result,
            {
                "[].uri" => insta::dynamic_redaction(redact_uri(path)),
                "." => insta::sorted_redaction(),
            }
        );
    };
}

#[test]
fn test_entry_definition() -> Result<()> {
    let mut server = ServerWrapper::spawn()?;
    server.initialize(json!({ "capabilities": {} }))?;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("foo.bib"),
            "languageId": "bibtex",
            "version": 1i32,
            "text": r#"@article{foo,}"#
        }
    }))?;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("bar.tex"),
            "languageId": "latex",
            "version": 1i32,
            "text": r#"\cite{foo}\addbibresource{foo.bib}"#
        }
    }))?;

    let result = server.request::<References>(json!({
        "textDocument": {
            "uri": server.uri("foo.bib")
        },
        "position": {
            "line": 0u32,
            "character": 11u32
        },
        "context": {
            "includeDeclaration": false
        }
    }))?;

    verify!(server, result);
    Ok(())
}

#[test]
fn test_entry_definition_include_declaration() -> Result<()> {
    let mut server = ServerWrapper::spawn()?;
    server.initialize(json!({ "capabilities": {} }))?;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("foo.bib"),
            "languageId": "bibtex",
            "version": 1i32,
            "text": r#"@article{foo,}"#
        }
    }))?;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("bar.tex"),
            "languageId": "latex",
            "version": 1i32,
            "text": r#"\cite{foo}\addbibresource{foo.bib}"#
        }
    }))?;

    let result = server.request::<References>(json!({
        "textDocument": {
            "uri": server.uri("foo.bib")
        },
        "position": {
            "line": 0u32,
            "character": 11u32
        },
        "context": {
            "includeDeclaration": true
        }
    }))?;

    verify!(server, result);
    Ok(())
}

#[test]
fn test_entry_reference() -> Result<()> {
    let mut server = ServerWrapper::spawn()?;
    server.initialize(json!({ "capabilities": {} }))?;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("foo.bib"),
            "languageId": "bibtex",
            "version": 1i32,
            "text": r#"@article{foo,}"#
        }
    }))?;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("bar.tex"),
            "languageId": "latex",
            "version": 1i32,
            "text": r#"\cite{foo}\addbibresource{foo.bib}"#
        }
    }))?;

    let result = server.request::<References>(json!({
        "textDocument": {
            "uri": server.uri("bar.tex")
        },
        "position": {
            "line": 0u32,
            "character": 8u32
        },
        "context": {
            "includeDeclaration": false
        }
    }))?;

    verify!(server, result);
    Ok(())
}

#[test]
fn test_entry_reference_include_declaration() -> Result<()> {
    let mut server = ServerWrapper::spawn()?;
    server.initialize(json!({ "capabilities": {} }))?;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("foo.bib"),
            "languageId": "bibtex",
            "version": 1i32,
            "text": r#"@article{foo,}"#
        }
    }))?;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("bar.tex"),
            "languageId": "latex",
            "version": 1i32,
            "text": r#"\cite{foo}\addbibresource{foo.bib}"#
        }
    }))?;

    let result = server.request::<References>(json!({
        "textDocument": {
            "uri": server.uri("bar.tex")
        },
        "position": {
            "line": 0u32,
            "character": 6u32
        },
        "context": {
            "includeDeclaration": true
        }
    }))?;

    verify!(server, result);
    Ok(())
}

#[test]
fn test_label_definition() -> Result<()> {
    let mut server = ServerWrapper::spawn()?;
    server.initialize(json!({ "capabilities": {} }))?;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("foo.tex"),
            "languageId": "latex",
            "version": 1i32,
            "text": r#"\label{foo}"#
        }
    }))?;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("bar.tex"),
            "languageId": "latex",
            "version": 1i32,
            "text": r#"\ref{foo}\input{foo.tex}"#
        }
    }))?;

    let result = server.request::<References>(json!({
        "textDocument": {
            "uri": server.uri("foo.tex")
        },
        "position": {
            "line": 0u32,
            "character": 8u32
        },
        "context": {
            "includeDeclaration": false
        }
    }))?;

    verify!(server, result);
    Ok(())
}

#[test]
fn test_label_definition_include_declaration() -> Result<()> {
    let mut server = ServerWrapper::spawn()?;
    server.initialize(json!({ "capabilities": {} }))?;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("foo.tex"),
            "languageId": "latex",
            "version": 1i32,
            "text": r#"\label{foo}"#
        }
    }))?;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("bar.tex"),
            "languageId": "latex",
            "version": 1i32,
            "text": r#"\ref{foo}\input{foo.tex}"#
        }
    }))?;

    let result = server.request::<References>(json!({
        "textDocument": {
            "uri": server.uri("foo.tex")
        },
        "position": {
            "line": 0u32,
            "character": 9u32
        },
        "context": {
            "includeDeclaration": true
        }
    }))?;

    verify!(server, result);
    Ok(())
}

#[test]
fn test_label_reference() -> Result<()> {
    let mut server = ServerWrapper::spawn()?;
    server.initialize(json!({ "capabilities": {} }))?;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("foo.tex"),
            "languageId": "latex",
            "version": 1i32,
            "text": r#"\label{foo}\input{bar.tex}"#
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
            "text": r#"\ref{foo}\input{bar.tex}"#
        }
    }))?;

    let result = server.request::<References>(json!({
        "textDocument": {
            "uri": server.uri("bar.tex")
        },
        "position": {
            "line": 0u32,
            "character": 7u32
        },
        "context": {
            "includeDeclaration": false
        }
    }))?;

    verify!(server, result);
    Ok(())
}

#[test]
fn test_label_reference_include_declaration() -> Result<()> {
    let mut server = ServerWrapper::spawn()?;
    server.initialize(json!({ "capabilities": {} }))?;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("foo.tex"),
            "languageId": "latex",
            "version": 1i32,
            "text": r#"\label{foo}\input{bar.tex}"#
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
            "text": r#"\ref{foo}\input{bar.tex}"#
        }
    }))?;

    let result = server.request::<References>(json!({
        "textDocument": {
            "uri": server.uri("bar.tex")
        },
        "position": {
            "line": 0u32,
            "character": 7u32
        },
        "context": {
            "includeDeclaration": true
        }
    }))?;

    verify!(server, result);
    Ok(())
}

#[test]
fn test_string_definition() -> Result<()> {
    let mut server = ServerWrapper::spawn()?;
    server.initialize(json!({ "capabilities": {} }))?;

    let text = r#"@string{foo = {Foo}}
@string{bar = {Bar}}
@article{baz, author = foo}"#;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("main.bib"),
            "languageId": "bibtex",
            "version": 1i32,
            "text": text
        }
    }))?;

    let result = server.request::<References>(json!({
        "textDocument": {
            "uri": server.uri("main.bib")
        },
        "position": {
            "line": 2u32,
            "character": 24u32
        },
        "context": {
            "includeDeclaration": false
        }
    }))?;

    verify!(server, result);
    Ok(())
}

#[test]
fn test_string_definition_include_declaration() -> Result<()> {
    let mut server = ServerWrapper::spawn()?;
    server.initialize(json!({ "capabilities": {} }))?;

    let text = r#"@string{foo = {Foo}}
@string{bar = {Bar}}
@article{baz, author = foo}"#;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("main.bib"),
            "languageId": "bibtex",
            "version": 1i32,
            "text": text
        }
    }))?;

    let result = server.request::<References>(json!({
        "textDocument": {
            "uri": server.uri("main.bib")
        },
        "position": {
            "line": 2u32,
            "character": 24u32
        },
        "context": {
            "includeDeclaration": true
        }
    }))?;

    verify!(server, result);
    Ok(())
}

#[test]
fn test_string_reference() -> Result<()> {
    let mut server = ServerWrapper::spawn()?;
    server.initialize(json!({ "capabilities": {} }))?;

    let text = r#"@string{foo = {Foo}}
@string{bar = {Bar}}
@article{baz, author = foo}"#;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("main.bib"),
            "languageId": "bibtex",
            "version": 1i32,
            "text": text
        }
    }))?;

    let result = server.request::<References>(json!({
        "textDocument": {
            "uri": server.uri("main.bib")
        },
        "position": {
            "line": 0u32,
            "character": 10u32
        },
        "context": {
            "includeDeclaration": false
        }
    }))?;

    verify!(server, result);
    Ok(())
}

#[test]
fn test_string_reference_include_declaration() -> Result<()> {
    let mut server = ServerWrapper::spawn()?;
    server.initialize(json!({ "capabilities": {} }))?;

    let text = r#"@string{foo = {Foo}}
@string{bar = {Bar}}
@article{baz, author = foo}"#;

    server.notify::<DidOpenTextDocument>(json!({
        "textDocument": {
            "uri": server.uri("main.bib"),
            "languageId": "bibtex",
            "version": 1i32,
            "text": text
        }
    }))?;

    let result = server.request::<References>(json!({
        "textDocument": {
            "uri": server.uri("main.bib")
        },
        "position": {
            "line": 0u32,
            "character": 10u32
        },
        "context": {
            "includeDeclaration": true
        }
    }))?;

    verify!(server, result);
    Ok(())
}
