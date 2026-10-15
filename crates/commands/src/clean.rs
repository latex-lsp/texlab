use std::{path::PathBuf, process::Stdio};

use anyhow::Result;
use base_db::{Document, Workspace, deps::ProjectRoot};

use crate::placeholders::replace_placeholders;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum CleanTarget {
    Auxiliary,
    Artifacts,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

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
        CleanCommand {
            executable,
            args,
            working_dir: PathBuf::from("."),
        }
    }

    #[test]
    fn clean_reports_nonzero_exit_and_accepts_success() {
        exit_command(0).run().unwrap();
        let error = exit_command(7).run().unwrap_err();
        assert!(error.to_string().contains("exited with status"));
    }
}

#[derive(Debug)]
pub struct CleanCommand {
    executable: String,
    args: Vec<String>,
    working_dir: PathBuf,
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
        let build = &workspace.config().build;

        let (config, out_dir) = match target {
            CleanTarget::Auxiliary => (&build.clean, &root.aux_dir),
            CleanTarget::Artifacts => (&build.clobber, &root.pdf_dir),
        };

        let out_dir = out_dir
            .to_file_path()
            .map_err(|()| anyhow::format_err!("directory '{out_dir}' is not a local file"))?
            .to_string_lossy()
            .into_owned();

        let working_dir = root.compile_dir.to_file_path().map_err(|()| {
            anyhow::format_err!("directory '{}' is not a local file", root.compile_dir)
        })?;

        let source_path = path.to_string_lossy().into_owned();
        let args = replace_placeholders(&config.args, &[('f', &source_path), ('o', &out_dir)]);

        Ok(Self {
            executable: config.program.clone(),
            args,
            working_dir,
        })
    }

    pub fn run(self) -> Result<()> {
        log::debug!("Cleaning output files: {} {:?}", self.executable, self.args);
        let status = std::process::Command::new(&self.executable)
            .args(&self.args)
            .current_dir(&self.working_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;

        if !status.success() {
            anyhow::bail!("{} exited with status {status}", self.executable);
        }

        Ok(())
    }
}
