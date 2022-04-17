use std::{
    fs, io,
    path::{Path, PathBuf},
    str::Lines,
};

use anyhow::Result;

use super::kpsewhich::{self, Resolver};

pub fn load_resolver() -> Result<Resolver> {
    let root_directories = kpsewhich::root_directories()?;
    let resolver = kpsewhich::parse_database(&root_directories, read_database)?;
    Ok(resolver)
}

const DATABASE_PATH: &str = "ls-R";

fn read_database(directory: &Path) -> Result<Vec<PathBuf>> {
    let file = directory.join(DATABASE_PATH);
    if !file.is_file() {
        return Ok(Vec::new());
    }

    let text = fs::read_to_string(file)?;
    let files = parse_database(text.lines())?;
    Ok(files)
}

fn parse_database(lines: Lines) -> io::Result<Vec<PathBuf>> {
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
    Ok(paths)
}
