use super::{
    compile,
    kpsewhich::{self, KpsewhichError, Resolver},
    Artifacts, CompileError, CompileParams, Distribution, DistributionKind,
};
use async_trait::async_trait;
use futures::lock::Mutex;
use std::{
    io, mem,
    path::{Path, PathBuf},
    str::Lines,
    sync::Arc,
};
use tokio::fs;

#[derive(Debug, Default)]
pub struct Texlive {
    resolver: Mutex<Arc<Resolver>>,
}

impl Texlive {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Distribution for Texlive {
    fn kind(&self) -> DistributionKind {
        DistributionKind::Texlive
    }

    async fn compile<'a>(&'a self, params: CompileParams<'a>) -> Result<Artifacts, CompileError> {
        compile(params).await
    }

    async fn load(&self) -> Result<(), KpsewhichError> {
        let root_directories = kpsewhich::root_directories().await?;
        let resolver = kpsewhich::parse_database(&root_directories, read_database).await?;
        mem::replace(&mut *self.resolver.lock().await, Arc::new(resolver));
        Ok(())
    }

    async fn resolver(&self) -> Arc<Resolver> {
        let resolver = self.resolver.lock().await;
        Arc::clone(&resolver)
    }
}

const DATABASE_PATH: &str = "ls-R";

async fn read_database(directory: &Path) -> Result<Vec<PathBuf>, KpsewhichError> {
    let file = directory.join(DATABASE_PATH);
    if !file.is_file() {
        return Ok(Vec::new());
    }

    let text = fs::read_to_string(file)
        .await
        .map_err(|_| KpsewhichError::NoDatabase)?;
    parse_database(text.lines()).map_err(|_| KpsewhichError::CorruptDatabase)
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
