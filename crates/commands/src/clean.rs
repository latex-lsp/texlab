use std::process::Stdio;

use anyhow::Result;
use base_db::{Document, Workspace};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum CleanTarget {
    Auxiliary,
    Artifacts,
}

#[derive(Debug)]
pub struct CleanCommand {
    executable: String,
    args: Vec<String>,
}

impl CleanCommand {
    pub fn new(workspace: &Workspace, document: &Document, target: CleanTarget) -> Result<Self> {
        let Some(path) = document.path.as_deref() else {
            anyhow::bail!("document '{}' is not a local file", document.uri)
        };

        let dir = workspace.current_dir(&document.dir);
        let dir = workspace
            .output_dir(&dir, workspace.config().build.log_dir.clone())
            .to_file_path()
            .unwrap();

        let flag = match target {
            CleanTarget::Auxiliary => "-c",
            CleanTarget::Artifacts => "-C",
        };

        let executable = String::from("latexmk");
        let args = vec![
            format!("-outdir={}", dir.display()),
            String::from(flag),
            path.display().to_string(),
        ];

        Ok(Self { executable, args })
    }

    pub fn run(self) -> Result<()> {
        log::debug!("Cleaning output files: {} {:?}", self.executable, self.args);
        let result = std::process::Command::new(self.executable)
            .args(self.args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        if let Err(why) = result {
            anyhow::bail!("failed to spawn process: {why}")
        }

        Ok(())
    }
}
