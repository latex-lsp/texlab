use std::{
    env,
    path::{Path, PathBuf},
};

use distro::FileNameDB;
use syntax::latexmkrc::LatexmkrcData;

mod v483 {
    use std::{path::Path, sync::Arc};

    use syntax::latexmkrc::LatexmkrcData;
    use tempfile::tempdir;

    use crate::latexmkrc::change_root;

    use super::{append_texinputs, parse_texinputs};

    pub fn parse_latexmkrc(input: &str, src_dir: &Path) -> std::io::Result<LatexmkrcData> {
        let temp_dir = tempdir()?;
        let non_existent_tex = temp_dir.path().join("NONEXISTENT.tex");
        std::fs::write(temp_dir.path().join(".latexmkrc"), &append_texinputs(input))?;

        // Run `latexmk -dir-report $TMPDIR/NONEXISTENT.tex` to obtain out_dir
        // and aux_dir values. We pass nonexistent file to prevent latexmk from
        // building anything, since we need this invocation only to extract the
        // -dir-report variables.
        //
        // In later versions, latexmk provides the -dir-report-only option and we
        // won't have to resort to this hack with NONEXISTENT.tex.
        let output = std::process::Command::new("latexmk")
            .arg("-dir-report")
            .arg(non_existent_tex)
            .current_dir(temp_dir.path())
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        let (file_name_db, mut lines) = parse_texinputs(&stdout, src_dir, temp_dir.path());

        let (aux_dir, out_dir) = lines.find_map(extract_dirs).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Normalized aux and out dir were not found in latexmk output",
            )
        })?;

        let aux_dir = change_root(src_dir, temp_dir.path(), &aux_dir);
        let out_dir = change_root(src_dir, temp_dir.path(), &out_dir);

        Ok(LatexmkrcData {
            aux_dir,
            out_dir,
            file_name_db: Arc::new(file_name_db),
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

        Some((String::from(aux_dir), String::from(out_dir)))
    }
}

mod v484 {
    use std::{path::Path, str::Lines, sync::Arc};

    use syntax::latexmkrc::LatexmkrcData;
    use tempfile::tempdir;

    use super::{append_texinputs, change_root, parse_texinputs};

    pub fn parse_latexmkrc(input: &str, src_dir: &Path) -> std::io::Result<LatexmkrcData> {
        let temp_dir = tempdir()?;
        std::fs::write(temp_dir.path().join(".latexmkrc"), &append_texinputs(input))?;

        // Create an empty dummy TeX file to let latexmk continue
        std::fs::write(temp_dir.path().join("dummy.tex"), "")?;

        // Run `latexmk -dir-report-only` to obtain out_dir and aux_dir values.
        let output = std::process::Command::new("latexmk")
            .arg("-dir-report-only")
            .current_dir(temp_dir.path())
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        let (file_name_db, lines) = parse_texinputs(&stdout, src_dir, temp_dir.path());

        let (aux_dir, out_dir) = extract_dirs(lines).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Normalized aux and out dir were not found in latexmk output",
            )
        })?;

        let aux_dir = change_root(src_dir, temp_dir.path(), &aux_dir);
        let out_dir = change_root(src_dir, temp_dir.path(), &out_dir);

        Ok(LatexmkrcData {
            aux_dir,
            out_dir,
            file_name_db: Arc::new(file_name_db),
        })
    }

    /// Extracts $aux_dir and $out_dir from lines of the form
    ///
    ///   Latexmk: Normalized aux dir and out dirs:
    ///    '$aux_dir', '$out_dir', [...]
    fn extract_dirs(lines: Lines) -> Option<(String, String)> {
        let mut it = lines
            .skip_while(|line| !line.starts_with("Latexmk: Normalized aux dir and out dirs:"))
            .nth(1)?
            .split(',');

        let aux_dir = it.next()?.trim().strip_prefix('\'')?.strip_suffix('\'')?;

        it.next(); // Skip the old 'outdir' option.

        let out_dir = it.next()?.trim().strip_prefix('\'')?.strip_suffix('\'')?;

        // Ensure there's no more data
        if it.next().is_some() {
            return None;
        }

        Some((String::from(aux_dir), String::from(out_dir)))
    }
}

pub fn parse_latexmkrc(input: &str, src_dir: &Path) -> std::io::Result<LatexmkrcData> {
    let output = std::process::Command::new("latexmk")
        .arg("--version")
        .output()?;

    let version = String::from_utf8(output.stdout)
        .ok()
        .as_ref()
        .and_then(|line| Some((line.find("Version")?, line)))
        .and_then(|(i, line)| line[i..].trim_end().strip_prefix("Version "))
        .and_then(versions::Versioning::new);

    let result = if version.map_or(false, |v| v >= versions::Versioning::new("4.84").unwrap()) {
        v484::parse_latexmkrc(input, src_dir)
    } else {
        v483::parse_latexmkrc(input, src_dir)
    };

    log::debug!("Latexmkrc parsing result: src_dir={src_dir:?}, output={result:?}");
    result
}

fn change_root(src_dir: &Path, tmp_dir: &Path, out_dir: impl AsRef<Path>) -> Option<String> {
    let out_dir = tmp_dir.join(out_dir);
    let relative_to_tmp = pathdiff::diff_paths(out_dir, tmp_dir)?;
    let relative_to_src = pathdiff::diff_paths(src_dir.join(relative_to_tmp), src_dir)?;
    if relative_to_src == PathBuf::new() {
        return None;
    }

    Some(relative_to_src.to_str()?.to_string())
}

fn append_texinputs(latexmkrc: &str) -> String {
    let mut output = String::new();
    output.push_str(latexmkrc);
    output.push('\n');
    output.push_str(r#"print "TEXINPUTS=" . $ENV{'TEXINPUTS'} . "\n";"#);
    output.push_str(r#"print "BIBINPUTS=" . $ENV{'BIBINPUTS'} . "\n";"#);
    output
}

fn parse_texinputs<'a>(
    stdout: &'a str,
    src_dir: &Path,
    tmp_dir: &Path,
) -> (FileNameDB, std::str::Lines<'a>) {
    let mut paths = Vec::new();
    let mut file_name_db = FileNameDB::default();

    let mut lines = stdout.lines();
    while let Some(line) = lines.next() {
        if let Some(inputs) = line.strip_prefix("TEXINPUTS=") {
            paths.extend(env::split_paths(inputs));
        } else if let Some(inputs) = line.strip_prefix("BIBINPUTS=") {
            paths.extend(env::split_paths(inputs));
            break;
        }
    }

    for path in paths
        .into_iter()
        .filter_map(|path| change_root(src_dir, tmp_dir, path))
        .map(|path| src_dir.join(shellexpand::tilde(&path).as_ref()))
    {
        eprintln!("Adding path: {path:?}");

        file_name_db.read_dir(path);
    }

    eprintln!("File name db: {file_name_db:?}");

    (file_name_db, lines)
}
