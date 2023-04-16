use std::{
    io::{BufReader, Read},
    path::{Path, PathBuf},
    process::{ExitStatus, Stdio},
    thread::{self, JoinHandle},
};

use anyhow::Result;
use base_db::Workspace;
use base_feature::replace_placeholders;
use bstr::io::BufReadExt;
use crossbeam_channel::Sender;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Document \"{0}\" was not found")]
    NotFound(Url),

    #[error("Document \"{0}\" does not exist on the local file system")]
    NotLocal(Url),

    #[error("Unable to run compiler: {0}")]
    Compile(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct BuildCommand {
    program: String,
    args: Vec<String>,
    working_dir: PathBuf,
}

impl BuildCommand {
    pub fn new(workspace: &Workspace, uri: &Url) -> Result<Self, BuildError> {
        let Some(document) = workspace.lookup(uri) else {
            return Err(BuildError::NotFound(uri.clone()));
        };

        let document = workspace
            .parents(document)
            .into_iter()
            .next()
            .unwrap_or(document);

        let Some(path) = document.path.as_deref().and_then(Path::to_str) else {
            return Err(BuildError::NotLocal(document.uri.clone()));
        };

        let config = &workspace.config().build;
        let program = config.program.clone();
        let args = replace_placeholders(&config.args, &[('f', path)]);

        let Ok(working_dir) = workspace.current_dir(&document.dir).to_file_path() else {
            return Err(BuildError::NotLocal(document.uri.clone()));
        };

        Ok(Self {
            program,
            args,
            working_dir,
        })
    }

    pub fn run(self, sender: Sender<String>) -> Result<ExitStatus, BuildError> {
        log::debug!(
            "Spawning compiler {} {:#?} in directory {}",
            self.program,
            self.args,
            self.working_dir.display()
        );

        let mut process = std::process::Command::new(&self.program)
            .args(self.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(&self.working_dir)
            .spawn()?;

        track_output(process.stderr.take().unwrap(), sender.clone());
        track_output(process.stdout.take().unwrap(), sender);

        let status = process.wait();
        Ok(status?)
    }
}

fn track_output(
    output: impl Read + Send + 'static,
    sender: Sender<String>,
) -> JoinHandle<std::io::Result<()>> {
    let mut reader = BufReader::new(output);
    thread::spawn(move || {
        reader.for_byte_line(|line| {
            let text = String::from_utf8_lossy(line).into_owned();
            let _ = sender.send(text);
            Ok(true)
        })
    })
}
