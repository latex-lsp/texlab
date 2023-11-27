use std::io::Write;

use syntax::latexmkrc::LatexmkrcData;
use tempfile::tempdir;

pub fn parse_latexmkrc(_input: &str) -> std::io::Result<LatexmkrcData> {
    let temp_dir = tempdir()?;
    let non_existent_tex = temp_dir.path().join("NONEXISTENT.tex");

    // Run `latexmk -dir-report $TMPDIR/NONEXISTENT.tex` to obtain out_dir
    // and aux_dir values. We pass nonexistent file to prevent latexmk from
    // building anything, since we need this invocation only to extract the
    // -dir-report variables.
    //
    // In the future, latexmk plans to implement -dir-report-only option and we
    // won't have to resort to this hack with NONEXISTENT.tex.
    let output = std::process::Command::new("latexmk")
        .arg("-dir-report")
        .arg(non_existent_tex)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    let (aux_dir, out_dir) = stdout
        .lines()
        .filter_map(extract_dirs)
        .next()
        .expect("Normalized aux and out dir were not found in latexmk output");

    Ok(LatexmkrcData {
        aux_dir: Some(aux_dir),
        out_dir: Some(out_dir),
    })
}

/// Extracts $aux_dir and $out_dir from lines of the form
///
///   Latexmk: Normalized aux dir and out dir: '$aux_dir', '$out_dir'
fn extract_dirs(line: &str) -> Option<(String, String)> {
    let mut it = line
        .strip_prefix("Latexmk: Normalized aux dir and out dir: ")?
        .split(", ");

    let aux_dir = it.next()?.strip_prefix('\'')?.strip_suffix('\'')?;
    let out_dir = it.next()?.strip_prefix('\'')?.strip_suffix('\'')?;

    // Ensure there's no more data
    if it.next().is_some() {
        return None;
    }

    Some((
        String::from(aux_dir),
        String::from(out_dir),
    ))
}
