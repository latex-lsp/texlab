use std::path::PathBuf;

use distro::Language;
use itertools::Itertools;
use rustc_hash::FxHashSet;

use crate::Workspace;

use super::ProjectRoot;

pub fn watch<T, C>(
    workspace: &mut Workspace,
    watcher: &mut notify_debouncer_full::Debouncer<T, C>,
    watched_dirs: &mut FxHashSet<PathBuf>,
) where
    T: notify::Watcher,
    C: notify_debouncer_full::FileIdCache,
{
    let roots = workspace
        .iter()
        .filter_map(|document| document.dir.as_ref())
        .filter(|dir| dir.scheme() == "file")
        .unique()
        .map(|dir| ProjectRoot::walk_and_find(workspace, dir));

    for root in roots {
        for uri in [&root.src_dir, &root.aux_dir, &root.log_dir, &root.pdf_dir] {
            if let Ok(path) = uri.to_file_path() {
                if watched_dirs.insert(path.clone()) {
                    let _ = watcher.watch(&path, notify::RecursiveMode::NonRecursive);
                }
            }
        }
    }
}

pub fn discover(workspace: &mut Workspace, checked_paths: &mut FxHashSet<PathBuf>) {
    loop {
        let mut changed = false;
        changed |= discover_parents(workspace, checked_paths);
        changed |= discover_children(workspace, checked_paths);
        if !changed {
            break;
        }
    }
}

fn discover_parents(workspace: &mut Workspace, checked_paths: &mut FxHashSet<PathBuf>) -> bool {
    let dirs = workspace
        .iter()
        .filter(|document| document.language != Language::Bib)
        .filter_map(|document| document.path.as_deref())
        .flat_map(|path| path.ancestors().skip(1))
        .filter(|path| workspace.contains(path))
        .map(|path| path.to_path_buf())
        .collect::<FxHashSet<_>>();

    let mut changed = false;
    for dir in dirs {
        if workspace
            .iter()
            .filter(|document| matches!(document.language, Language::Root | Language::Tectonic))
            .filter_map(|document| document.path.as_deref())
            .filter_map(|path| path.parent())
            .any(|marker| dir.starts_with(marker))
        {
            continue;
        }

        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };

        for file in entries
            .flatten()
            .filter(|entry| entry.file_type().map_or(false, |type_| type_.is_file()))
            .map(|entry| entry.path())
        {
            let Some(lang) = Language::from_path(&file) else {
                continue;
            };

            if !matches!(
                lang,
                Language::Tex | Language::Root | Language::Tectonic | Language::Latexmkrc
            ) {
                continue;
            }

            if workspace.lookup_file(&file).is_none() && file.exists() {
                changed |= workspace.load(&file, lang).is_ok();
                checked_paths.insert(file);
            }
        }
    }

    changed
}

fn discover_children(workspace: &mut Workspace, checked_paths: &mut FxHashSet<PathBuf>) -> bool {
    let files = workspace
        .graphs()
        .values()
        .flat_map(|graph| graph.missing.iter())
        .filter(|uri| uri.scheme() == "file")
        .flat_map(|uri| uri.to_file_path())
        .collect::<FxHashSet<_>>();

    let mut changed = false;
    for file in files {
        let language = Language::from_path(&file).unwrap_or(Language::Tex);

        if workspace.lookup_file(&file).is_none() && file.exists() {
            changed |= workspace.load(&file, language).is_ok();
            checked_paths.insert(file);
        }
    }

    changed
}
