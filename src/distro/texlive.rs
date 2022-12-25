use std::{
    fs,
    path::{Path, PathBuf},
    str::Lines,
};

use anyhow::Result;

const DATABASE_PATH: &str = "ls-R";

pub(super) fn read_database(directory: &Path) -> Result<Vec<PathBuf>> {
    let file = directory.join(DATABASE_PATH);
    if !file.is_file() {
        return Ok(Vec::new());
    }

    let text = fs::read_to_string(file)?;
    let files = parse_database(text.lines());
    Ok(files)
}

fn parse_database(lines: Lines) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut directory = "";

    for line in lines.filter(|x| !x.trim().is_empty() && !x.starts_with('%')) {
        if let Some(line) = line.strip_suffix(':') {
            directory = line;
        } else {
            let path = PathBuf::from(directory).join(line);
            paths.push(path);
        }
    }

    paths
}
