use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Stdio,
};

use anyhow::Result;
use base_db::Workspace;
use base_feature::replace_placeholders;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum ForwardSearchError {
    #[error("Forward search is not configured")]
    Unconfigured,

    #[error("Document \"{0}\" does not exist on the local file system")]
    NotLocal(Url),

    #[error("Document \"{0}\" has an invalid file path")]
    InvalidPath(Url),

    #[error("TeX document \"{0}\" not found")]
    TexNotFound(Url),

    #[error("PDF document \"{0}\" not found")]
    PdfNotFound(PathBuf),

    #[error("Unable to launch PDF viewer: {0}")]
    LaunchViewer(#[from] std::io::Error),
}

pub struct ForwardSearch {
    program: String,
    args: Vec<String>,
}

impl ForwardSearch {
    pub fn new(
        workspace: &Workspace,
        uri: &Url,
        line: Option<u32>,
    ) -> Result<Self, ForwardSearchError> {
        let Some(config) = &workspace.config().synctex else {
            return Err(ForwardSearchError::Unconfigured);
        };

        let Some(child) = workspace.lookup(uri) else {
            return Err(ForwardSearchError::TexNotFound(uri.clone()));
        };

        let parents = workspace.parents(child);
        let parent = parents.into_iter().next().unwrap_or(child);
        if parent.uri.scheme() != "file" {
            return Err(ForwardSearchError::NotLocal(parent.uri.clone()));
        }

        let dir = workspace.current_dir(&parent.dir);
        let dir = workspace.output_dir(&dir).to_file_path().unwrap();

        let Some(tex_path) = &child.path else {
            return Err(ForwardSearchError::InvalidPath(child.uri.clone()));
        };

        let Some(pdf_path) = parent.path
            .as_deref()
            .and_then(Path::file_stem)
            .and_then(OsStr::to_str)
            .map(|stem| dir.join(format!("{stem}.pdf"))) else 
        {
            return Err(ForwardSearchError::InvalidPath(parent.uri.clone()));
        };

        if !pdf_path.exists() {
            return Err(ForwardSearchError::PdfNotFound(pdf_path.clone()));
        }

        let tex_path = tex_path.to_string_lossy().into_owned();
        let pdf_path = pdf_path.to_string_lossy().into_owned();
        let line = line.unwrap_or_else(|| child.line_index.line_col(child.cursor).line);
        let line = (line + 1).to_string();

        let program = config.program.clone();
        let args = replace_placeholders(
            &config.args,
            &[('f', &tex_path), ('p', &pdf_path), ('l', &line)],
        );

        Ok(Self { program, args })
    }
}

impl ForwardSearch {
    pub fn run(self) -> Result<(), ForwardSearchError> {
        log::debug!("Executing forward search: {} {:?}", self.program, self.args);

        std::process::Command::new(self.program)
            .args(self.args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;

        Ok(())
    }
}
