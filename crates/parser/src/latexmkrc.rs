use std::str::Lines;

use syntax::latexmkrc::LatexmkrcData;

pub fn parse_latexmkrc(_input: &str) -> std::io::Result<LatexmkrcData> {

    // Run `latexmk -dir-report-only` to obtain out_dir and aux_dir values.
    let output = std::process::Command::new("latexmk")
        .arg("-dir-report-only")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    let (aux_dir, out_dir) =  extract_dirs(stdout.lines()).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Normalized aux and out dir were not found in latexmk output",
        )
    })?;

    Ok(LatexmkrcData {
        aux_dir: Some(aux_dir),
        out_dir: Some(out_dir),
    })
}

/// Extracts $aux_dir and $out_dir from lines of the form
///
///   Latexmk: Normalized aux dir and out dirs:
///    '$aux_dir', '$out_dir', [...]
fn extract_dirs(lines: Lines) -> Option<(String, String)> {
    let mut it =
        lines.skip_while(|line| {
            !line.starts_with("Latexmk: Normalized aux dir and out dirs:")
        })
        .nth(1)?
        .split(",");

    let aux_dir = it.next()?.trim().strip_prefix('\'')?.strip_suffix('\'')?;

    it.next(); // Skip the old 'outdir' option.

    let out_dir = it.next()?.trim().strip_prefix('\'')?.strip_suffix('\'')?;

    // Ensure there's no more data
    if it.next().is_some() {
        return None;
    }

    Some((String::from(aux_dir), String::from(out_dir)))
}
