use std::fs;

use anyhow::Result;
use lsp_types::ClientCapabilities;

use crate::common::ServerTester;

#[test]
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

    let uri = server.open("level1/level2/level3/d.tex", "d", "latex", false)?;
    server.complete(uri, 0, 0)?;

    let diagnostics_by_uri = server.diagnostics_by_uri.lock().unwrap();
    assert!(diagnostics_by_uri
        .iter()
        .all(|(uri, _)| !uri.as_str().ends_with("a.tex")));

    Ok(())
}
