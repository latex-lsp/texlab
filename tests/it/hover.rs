use anyhow::Result;
use insta::assert_json_snapshot;
use lsp_types::ClientCapabilities;

use crate::common::ServerTester;

#[test]
fn test_empty_bibtex_document() -> Result<()> {
    let server = ServerTester::launch_new_instance()?;
    server.initialize(ClientCapabilities::default(), None)?;
    let uri = server.open("main.bib", "", "bibtex", false)?;
    assert_json_snapshot!(server.hover(uri, 0, 0)?);
    Ok(())
}

#[test]
fn test_empty_labtex_document() -> Result<()> {
    let server = ServerTester::launch_new_instance()?;
    server.initialize(ClientCapabilities::default(), None)?;
    let uri = server.open("main.tex", "", "latex", false)?;
    assert_json_snapshot!(server.hover(uri, 0, 0)?);
    Ok(())
}

#[test]
fn test_label_theorem_child_file() -> Result<()> {
    let server = ServerTester::launch_new_instance()?;
    server.initialize(ClientCapabilities::default(), None)?;
    let uri = server.open(
        "main.tex",
        r#"
            \documentclass{article}
            \newtheorem{lemma}{Lemma}
            \include{child}
            \ref{thm:foo}
        "#,
        "latex",
        false,
    )?;
    server.open(
        "child.tex",
        r#"
            \begin{lemma}\label{thm:foo}
                1 + 1 = 2
            \end{lemma}
        "#,
        "latex",
        false,
    )?;

    assert_json_snapshot!(server.hover(uri, 3, 8)?);
    Ok(())
}

#[test]
fn test_label_theorem_child_file_mumber() -> Result<()> {
    let server = ServerTester::launch_new_instance()?;
    server.initialize(ClientCapabilities::default(), None)?;
    let uri = server.open(
        "main.tex",
        r#"
            \documentclass{article}
            \newtheorem{lemma}{Lemma}
            \include{child}
            \ref{thm:foo}
        "#,
        "latex",
        false,
    )?;
    server.open(
        "child.tex",
        r#"
            \begin{lemma}[Foo]\label{thm:foo}
                1 + 1 = 2
            \end{lemma}
        "#,
        "latex",
        false,
    )?;
    server.open(
        "child.aux",
        r#"\newlabel{thm:foo}{{1}{1}{Foo}{lemma.1}{}}"#,
        "latex",
        false,
    )?;

    assert_json_snapshot!(server.hover(uri, 3, 8)?);
    Ok(())
}
