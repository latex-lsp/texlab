use std::io::Write;

use syntax::latexmkrc::LatexmkrcData;
use tempfile::tempdir;

/// Extra section at the bottom of the latexmkrc file to print the values of $aux_dir and $log_dir.
const EXTRA: &str = r#"
print "texlab:aux_dir=" . $aux_dir . "\n";
print "texlab:out_dir=" . $out_dir . "\n";
exit 0; # Don't build the document"#;

pub fn parse_latexmkrc(input: &str) -> std::io::Result<LatexmkrcData> {
    let temp_dir = tempdir()?;
    let rc_path = temp_dir.path().join("latexmkrc");
    let mut rc_file = std::fs::File::create(&rc_path)?;
    rc_file.write_all(input.as_bytes())?;
    rc_file.write_all(EXTRA.as_bytes())?;
    drop(rc_file);

    let output = std::process::Command::new("latexmk")
        .arg("-r")
        .arg(rc_path)
        .output()?;

    let mut result = LatexmkrcData::default();
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        result.aux_dir = result
            .aux_dir
            .or_else(|| extract_dir(line, "texlab:aux_dir="));

        result.out_dir = result
            .out_dir
            .or_else(|| extract_dir(line, "texlab:out_dir="));
    }

    Ok(result)
}

fn extract_dir(line: &str, key: &str) -> Option<String> {
    line.strip_prefix(key)
        .filter(|path| !path.is_empty())
        .map(String::from)
}
