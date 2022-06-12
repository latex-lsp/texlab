use std::process::{Command, Stdio};

use anyhow::Result;

#[test]
fn rustfmt() -> Result<()> {
    let success = Command::new("cargo")
        .args(&["fmt", "--check"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?
        .success();

    assert!(success);
    Ok(())
}
