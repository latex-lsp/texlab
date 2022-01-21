use std::{fs, thread, time::Duration};

use anyhow::Result;
use lsp_types::ClientCapabilities;

use crate::common::ServerTester;

#[test]
#[cfg(feature = "completion")]
fn test_408_parent_expansion() -> Result<()> {
    let server = ServerTester::launch_new_instance()?;
    server.initialize(ClientCapabilities::default(), None)?;
    let root = server.directory.path();

    let level1 = root.join("level1");
    let level2 = level1.join("level2");
    let level3 = level2.join("level3");
    fs::create_dir_all(&level3)?;
    fs::write(level3.join("d.tex"), "d")?;
    fs::write(
        level2.join("c.tex"),
        r#"\documentclass{subfiles}\begin{document}\include{level3/d}\end{document}"#,
    )?;
    fs::write(
        level1.join("b.tex"),
        r#"\documentclass{article}\begin{document}\include{level2/c}\end{document}"#,
    )?;
    fs::write(root.join("a.tex"), "}")?;

    thread::sleep(Duration::from_millis(300));

    let uri = server.open("level1/level2/level3/d.tex", "d", "latex", false)?;
    server.complete(uri, 0, 0)?;

    let diagnostics_by_uri = server.diagnostics_by_uri.lock().unwrap();
    assert!(diagnostics_by_uri
        .iter()
        .all(|(uri, _)| !uri.as_str().ends_with("a.tex")));

    Ok(())
}

#[test]
#[cfg(feature = "completion")]
fn test_510_completion_with_unmatched_braces() -> Result<()> {
    use insta::assert_debug_snapshot;

    let server = ServerTester::launch_new_instance()?;
    server.initialize(ClientCapabilities::default(), None)?;

    let uri = server.open(
        "main.tex",
        "\\label{eq:foo}\n\\ref{eq is a \\emph{useful} identity.",
        "latex",
        false,
    )?;

    assert_debug_snapshot!(server.complete(uri, 1, 7)?);

    Ok(())
}

#[test]
#[cfg(feature = "completion")]
fn test_540_subimport_link() -> Result<()> {
    let server = ServerTester::launch_new_instance()?;
    server.initialize(ClientCapabilities::default(), None)?;

    server.open("stuff.tex", "\\usepackage{lipsum}", "latex", false)?;
    let uri = server.open("main.tex", "\\subimport{}{stuff}\n\\lipsu", "latex", false)?;

    let success = server
        .complete(uri, 1, 4)?
        .items
        .into_iter()
        .any(|item| item.label == "lipsum");

    assert!(success);

    Ok(())
}
