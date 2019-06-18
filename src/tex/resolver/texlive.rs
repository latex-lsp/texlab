use super::{Error, Result};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str::Lines;

pub const DATABASE_PATH: &'static str = "ls-R";

pub fn read_database(directory: &Path) -> Result<Vec<PathBuf>> {
    let file = directory.join(DATABASE_PATH);
    if !file.is_file() {
        return Ok(Vec::new());
    }

    let text = fs::read_to_string(file).expect("Could not read ls-R file");
    parse_database(text.lines()).map_err(|_| Error::CorruptFileDatabase)
}

fn parse_database(lines: Lines) -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let mut directory = "";

    for line in lines.filter(|x| !x.trim().is_empty() && !x.starts_with('%')) {
        if line.ends_with(':') {
            directory = &line[..line.len() - 1];
        } else {
            let path = PathBuf::from(directory).join(line);
            paths.push(path);
        }
    }

    Ok(paths)
}
