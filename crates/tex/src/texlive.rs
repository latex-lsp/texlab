use super::compile::*;
use super::kpsewhich;
use super::{Distribution, DistributionKind, LoadError, Resolver};
use futures::lock::Mutex;
use futures_boxed::boxed;
use std::fs;
use std::io;
use std::mem;
use std::path::{Path, PathBuf};
use std::str::Lines;
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct Texlive {
    resolver: Mutex<Arc<Resolver>>,
}

impl Texlive {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Distribution for Texlive {
    fn kind(&self) -> DistributionKind {
        DistributionKind::Texlive
    }

    fn supports_format(&self, format: Format) -> bool {
        match format {
            Format::Latex | Format::Pdflatex => true,
            Format::Xelatex | Format::Lualatex => true,
        }
    }

    #[boxed]
    async fn load(&self) -> Result<(), LoadError> {
        let resolver = Arc::new(kpsewhich::parse_database(read_database).await?);
        let mut resolver_guard = self.resolver.lock().await;
        mem::replace(&mut *resolver_guard, resolver);
        Ok(())
    }

    #[boxed]
    async fn resolver(&self) -> Arc<Resolver> {
        let resolver = self.resolver.lock().await;
        Arc::clone(&resolver)
    }
}

const DATABASE_PATH: &'static str = "ls-R";

fn read_database(directory: &Path) -> Result<Vec<PathBuf>, LoadError> {
    let file = directory.join(DATABASE_PATH);
    if !file.is_file() {
        return Ok(Vec::new());
    }

    let text = fs::read_to_string(file).expect("Could not read ls-R file");
    parse_database(text.lines()).map_err(|_| LoadError::CorruptFileDatabase)
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
