use std::{borrow::Cow, path::PathBuf};

use url::Url;

use crate::deps::HOME_DIR;

pub fn expand_relative_path(
    path: &str,
    current_dir: &Url,
    workspace_folders: &[PathBuf],
) -> Result<Url, url::ParseError> {
    let home_dir = HOME_DIR.as_ref().and_then(|dir| dir.to_str());

    let workspace_folder = workspace_folders.iter().find_map(|folder| {
        let current_dir = current_dir.to_file_path().ok()?;
        if current_dir.starts_with(folder) {
            Some(folder.to_str()?)
        } else {
            None
        }
    });

    let expand_var = |variable: &str| match variable {
        "userHome" => home_dir.map(Cow::Borrowed),
        "workspaceFolder" => Some(Cow::Borrowed(workspace_folder.unwrap_or("."))),
        _ => std::env::var(variable).ok().map(Cow::Owned),
    };

    let path = shellexpand::full_with_context_no_errors(&path, || home_dir, expand_var);
    current_dir.join(&path)
}
