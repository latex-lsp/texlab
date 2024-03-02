use std::{path::PathBuf, process::Stdio};

use anyhow::Result;
use base_db::{
    deps::{self, ProjectRoot},
    Document, Workspace,
};
use thiserror::Error;
use url::Url;

use crate::placeholders::replace_placeholders;

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

#[derive(Debug)]
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
        log::debug!("[FwdSearch] Preparing forward search: document={uri}, line={line:#?}");
        let synctex_config = workspace
            .config()
            .synctex
            .as_ref()
            .ok_or(ForwardSearchError::Unconfigured)?;

        log::debug!("[FwdSearch] synctex_config={:?}", synctex_config);

        let child = workspace
            .lookup(uri)
            .ok_or_else(|| ForwardSearchError::TexNotFound(uri.clone()))?;

        let parent = deps::parents(workspace, child)
            .into_iter()
            .next()
            .unwrap_or(child);

        log::debug!("[FwdSearch] root_document={}", parent.uri,);

        let pdf_path = Self::find_pdf(workspace, parent)?;
        let pdf_path = pdf_path.to_string_lossy().into_owned();
        let tex_path = child
            .path
            .as_deref()
            .ok_or_else(|| ForwardSearchError::InvalidPath(child.uri.clone()))?;
        let tex_path = tex_path.to_string_lossy().into_owned();

        let line = line.unwrap_or(child.cursor.line);
        let line = (line + 1).to_string();

        let program = synctex_config.program.clone();
        let args = replace_placeholders(
            &synctex_config.args,
            &[('f', &tex_path), ('p', &pdf_path), ('l', &line)],
        );

        Ok(Self { program, args })
    }

    fn find_pdf(workspace: &Workspace, document: &Document) -> Result<PathBuf, ForwardSearchError> {
        let root = ProjectRoot::walk_and_find(workspace, &document.dir);

        log::debug!("[FwdSearch] root={root:#?}");

        let pdf_dir = root
            .pdf_dir
            .to_file_path()
            .map_err(|()| ForwardSearchError::InvalidPath(document.uri.clone()))?;

        let pdf_name_override = workspace.config().build.output_filename.clone();
        log::debug!("[FwdSearch] pdf_name_override={pdf_name_override:?}");

        let pdf_name = pdf_name_override
            .or_else(|| {
                let stem = document.path.as_ref()?.file_stem()?;
                Some(format!("{}.pdf", stem.to_string_lossy()))
            })
            .ok_or_else(|| ForwardSearchError::InvalidPath(document.uri.clone()))?;

        let pdf_path = pdf_dir.join(&pdf_name);
        let pdf_exists = pdf_path.exists();

        log::debug!("[FwdSearch] pdf_path={pdf_path:?}, pdf_exists={pdf_exists}");
        if !pdf_exists {
            return Err(ForwardSearchError::PdfNotFound(pdf_path));
        }

        Ok(pdf_path)
    }
}

impl ForwardSearch {
    pub fn run(self) -> Result<(), ForwardSearchError> {
        log::debug!(
            "[FwdSearch] Executing command: {} {:?}",
            self.program,
            self.args
        );

        std::process::Command::new(self.program)
            .args(self.args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;

        Ok(())
    }
}
