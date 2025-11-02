use std::sync::Arc;

use distro::FileNameDB;
use url::Url;

use crate::{DocumentData, Workspace, util};

use super::graph::HOME_DIR;

#[derive(Clone)]
pub struct ProjectRoot {
    pub compile_dir: Url,
    pub src_dir: Url,
    pub aux_dir: Url,
    pub log_dir: Url,
    pub pdf_dir: Url,
    pub additional_files: Vec<Url>,
    pub file_name_db: Arc<FileNameDB>,
}

impl ProjectRoot {
    pub fn walk_and_find(workspace: &Workspace, dir: &Url) -> Self {
        let home_dir = HOME_DIR
            .as_deref()
            .and_then(|path| Url::from_directory_path(path).ok());

        let mut current = dir.clone();
        loop {
            let root = Self::from_rootfile(workspace, &current)
                .or_else(|| Self::from_tectonic(workspace, &current))
                .or_else(|| Self::from_latexmkrc(workspace, &current));

            if let Some(root) = root {
                break root;
            }

            let Ok(parent) = current.join("..") else {
                break Self::from_config(workspace, dir);
            };

            if current == parent || Some(&parent) == home_dir.as_ref() {
                break Self::from_config(workspace, dir);
            }

            current = parent;
        }
    }

    pub fn from_tectonic(workspace: &Workspace, dir: &Url) -> Option<Self> {
        let exists = workspace
            .iter()
            .filter(|document| document.dir.as_ref() == Some(dir))
            .any(|document| matches!(document.data, DocumentData::Tectonic));

        if !exists {
            return None;
        }

        let compile_dir = dir.clone();
        let src_dir = dir.join("src/").unwrap();
        let out_dir = dir.join("build/").unwrap();
        let aux_dir = out_dir.clone();
        let log_dir = out_dir.clone();
        let pdf_dir = out_dir;
        let additional_files = vec![
            src_dir.join("_preamble.tex").unwrap(),
            src_dir.join("_postamble.tex").unwrap(),
        ];

        Some(Self {
            compile_dir,
            src_dir,
            aux_dir,
            log_dir,
            pdf_dir,
            additional_files,
            file_name_db: Default::default(),
        })
    }

    pub fn from_latexmkrc(workspace: &Workspace, dir: &Url) -> Option<Self> {
        let config = workspace.config();
        let rcfile = workspace
            .iter()
            .filter(|document| document.dir.as_ref() == Some(dir))
            .find_map(|document| document.data.as_latexmkrc())?;

        let compile_dir = dir.clone();
        let src_dir = dir.clone();

        let aux_dir_rc = rcfile
            .aux_dir
            .as_ref()
            .and_then(|path| append_dir(dir, path, workspace).ok());

        let out_dir_rc = rcfile
            .out_dir
            .as_ref()
            .and_then(|path| append_dir(dir, path, workspace).ok());

        let aux_dir = aux_dir_rc
            .clone()
            .or_else(|| append_dir(dir, &config.build.aux_dir, workspace).ok())
            .unwrap_or_else(|| dir.clone());

        let log_dir = aux_dir_rc
            .or_else(|| append_dir(dir, &config.build.log_dir, workspace).ok())
            .unwrap_or_else(|| dir.clone());

        let pdf_dir = out_dir_rc
            .or_else(|| append_dir(dir, &config.build.pdf_dir, workspace).ok())
            .unwrap_or_else(|| dir.clone());

        let additional_files = vec![];

        let file_name_db = Arc::clone(&rcfile.file_name_db);

        Some(Self {
            compile_dir,
            src_dir,
            aux_dir,
            log_dir,
            pdf_dir,
            additional_files,
            file_name_db,
        })
    }

    pub fn from_rootfile(workspace: &Workspace, dir: &Url) -> Option<Self> {
        let exists = workspace
            .iter()
            .filter(|document| document.dir.as_ref() == Some(dir))
            .any(|document| matches!(document.data, DocumentData::Root));

        if !exists {
            return None;
        }

        Some(Self::from_config(workspace, dir))
    }

    pub fn from_config(workspace: &Workspace, dir: &Url) -> Self {
        let compile_dir = dir.clone();
        let src_dir = dir.clone();
        let config = workspace.config();
        let aux_dir =
            append_dir(dir, &config.build.aux_dir, workspace).unwrap_or_else(|_| dir.clone());
        let log_dir =
            append_dir(dir, &config.build.log_dir, workspace).unwrap_or_else(|_| dir.clone());
        let pdf_dir =
            append_dir(dir, &config.build.pdf_dir, workspace).unwrap_or_else(|_| dir.clone());
        let additional_files = vec![];

        Self {
            compile_dir,
            src_dir,
            aux_dir,
            log_dir,
            pdf_dir,
            additional_files,
            file_name_db: Default::default(),
        }
    }
}

fn append_dir(dir: &Url, path: &str, workspace: &Workspace) -> Result<Url, url::ParseError> {
    let mut path = String::from(path);
    if !path.ends_with('/') {
        path.push('/');
    }

    util::expand_relative_path(&path, dir, workspace.folders())
}

impl std::fmt::Debug for ProjectRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProjectRoot")
            .field("compile_dir", &self.compile_dir.as_str())
            .field("src_dir", &self.src_dir.as_str())
            .field("aux_dir", &self.aux_dir.as_str())
            .field("log_dir", &self.log_dir.as_str())
            .field("pdf_dir", &self.pdf_dir.as_str())
            .field("additional_files", &self.additional_files)
            .field("file_name_db", &"...")
            .finish()
    }
}
