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
    parse_database(&directory, text.lines()).map_err(|_| Error::CorruptFileDatabase)
}

fn parse_database(root_directory: &Path, lines: Lines) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut directory = PathBuf::new();

    for line in lines.filter(|x| !x.trim().is_empty() && !x.starts_with('%')) {
        if line.ends_with(':') {
            let path = &line[..line.len() - 1];
            directory = root_directory.join(path);
        } else {
            let file = directory.join(line).canonicalize()?;
            if file.is_file() {
                files.push(file);
            }
        }
    }

    Ok(files)
}
