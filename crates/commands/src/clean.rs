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

        let base_dir = workspace.current_dir(&document.dir);

        let flag = match target {
            CleanTarget::Auxiliary => "-c",
            CleanTarget::Artifacts => "-C",
        };

        let out_dir = match target {
            CleanTarget::Auxiliary => workspace.aux_dir(&base_dir),
            CleanTarget::Artifacts => workspace.pdf_dir(&base_dir),
        };

        let out_dir_path = out_dir.to_file_path().unwrap();

        let executable = String::from("latexmk");
        let args = vec![
            format!("-outdir={}", out_dir_path.display()),
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
