use std::process::Stdio;

use anyhow::Result;
use base_db::{Document, Workspace, deps::ProjectRoot};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum CleanTarget {
    Auxiliary,
    Artifacts,
}

#[cfg(test)]
mod tests {
    use super::CleanCommand;

    fn exit_command(code: u8) -> CleanCommand {
        #[cfg(windows)]
        let (executable, args) = (
            String::from("cmd.exe"),
            vec![
                String::from("/d"),
                String::from("/c"),
                format!("exit {code}"),
            ],
        );
        #[cfg(not(windows))]
        let (executable, args) = (
            String::from("sh"),
            vec![String::from("-c"), format!("exit {code}")],
        );
        CleanCommand { executable, args }
    }

    #[test]
    fn clean_reports_nonzero_exit_and_accepts_success() {
        exit_command(0).run().unwrap();
        let error = exit_command(7).run().unwrap_err();
        assert!(error.to_string().contains("latexmk exited with status"));
    }
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

        let Some(document_dir) = &document.dir else {
            anyhow::bail!("document '{}' is not a local file", document.uri)
        };

        let root = ProjectRoot::walk_and_find(workspace, document_dir);

        let flag = match target {
            CleanTarget::Auxiliary => "-c",
            CleanTarget::Artifacts => "-C",
        };

        let out_dir = match target {
            CleanTarget::Auxiliary => root.aux_dir,
            CleanTarget::Artifacts => root.pdf_dir,
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
        let status = std::process::Command::new(self.executable)
            .args(self.args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;

        if !status.success() {
            anyhow::bail!("latexmk exited with status {status}")
        }

        Ok(())
    }
}
