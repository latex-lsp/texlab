use std::{
    fs::{self},
    path::{Path, PathBuf},
};

use base_db::{
    deps::{self, ProjectRoot},
    util, DocumentData, FeatureParams,
};
use rowan::{ast::AstNode, TextLen, TextRange};
use syntax::latex;

use crate::{
    util::{find_curly_group_word_list, CompletionBuilder},
    CompletionItem, CompletionItemData, CompletionParams,
};

pub fn complete_includes<'a>(
    params: &'a CompletionParams<'a>,
    builder: &mut CompletionBuilder<'a>,
) -> Option<()> {
    if params.feature.document.path.is_none() {
        return None;
    }

    let (cursor, group) = find_curly_group_word_list(params)?;

    let include = group.syntax().parent()?;
    let (include_extension, extensions): (bool, &[&str]) = match include.kind() {
        latex::PACKAGE_INCLUDE => (false, &["sty"]),
        latex::CLASS_INCLUDE => (false, &["cls"]),
        latex::LATEX_INCLUDE => {
            let include = latex::Include::cast(include.clone())?;
            (
                matches!(include.command()?.text(), "\\input" | "\\subfile"),
                &["tex"],
            )
        }
        latex::BIBLATEX_INCLUDE => (true, &["bib"]),
        latex::BIBTEX_INCLUDE => (false, &["bib"]),
        latex::GRAPHICS_INCLUDE => (true, &["pdf", "png", "jpg", "jpeg", "bmp"]),
        latex::SVG_INCLUDE => (true, &["svg"]),
        latex::INKSCAPE_INCLUDE => (true, &["pdf", "eps", "ps", "png"]),
        latex::VERBATIM_INCLUDE => (true, &[]),
        _ => return None,
    };

    let segment_range = if cursor.text.is_empty() {
        cursor.range
    } else {
        let start = cursor.range.end() - cursor.text.split('/').last()?.text_len();
        TextRange::new(start, cursor.range.end())
    };

    let segment_text = &params.feature.document.text[std::ops::Range::from(segment_range)];

    let mut dirs = vec![current_dir(&params.feature, &cursor.text, None)];
    if include.kind() == latex::GRAPHICS_INCLUDE {
        for document in &params.feature.project.documents {
            let DocumentData::Tex(data) = &document.data else {
                continue;
            };

            for graphics_path in &data.semantics.graphics_paths {
                dirs.push(current_dir(
                    &params.feature,
                    &cursor.text,
                    Some(graphics_path),
                ));
            }
        }
    }

    for entry in dirs
        .into_iter()
        .flatten()
        .filter_map(|dir| fs::read_dir(dir).ok())
        .flatten()
        .flatten()
    {
        let mut path = entry.path();

        let file_type = entry.file_type().ok()?;
        if file_type.is_file() && is_included(&path, extensions) {
            if !include_extension {
                remove_extension(&mut path);
            }

            let name = String::from(path.file_name()?.to_str()?);
            if let Some(score) = builder.matcher.score(&name, segment_text) {
                builder.items.push(CompletionItem::new_simple(
                    score,
                    segment_range,
                    CompletionItemData::File(name),
                ));
            }
        } else if file_type.is_dir() {
            let name = String::from(path.file_name()?.to_str()?);
            if let Some(score) = builder.matcher.score(&name, segment_text) {
                builder.items.push(CompletionItem::new_simple(
                    score,
                    segment_range,
                    CompletionItemData::Directory(name),
                ));
            }
        }
    }

    Some(())
}

fn current_dir(
    params: &FeatureParams,
    path_text: &str,
    graphics_path: Option<&str>,
) -> Option<PathBuf> {
    let workspace = &params.workspace;
    let parent = deps::parents(&workspace, params.document)
        .iter()
        .next()
        .map_or(params.document, Clone::clone);

    let root = ProjectRoot::walk_and_find(workspace, &parent.dir);

    let mut path = PathBuf::new();
    if let Some(graphics_path) = graphics_path {
        path.push(graphics_path);
    }

    if !path_text.is_empty() {
        path.push(path_text);
        if !path_text.ends_with('/') {
            path.pop();
        }
    }

    let current_dir =
        util::expand_relative_path(path.to_str()?, &root.src_dir, workspace.folders()).ok()?;

    current_dir.to_file_path().ok()
}

fn is_included(file: &Path, allowed_extensions: &[&str]) -> bool {
    allowed_extensions.is_empty()
        || file
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .map(str::to_lowercase)
            .map(|ext| allowed_extensions.contains(&ext.as_str()))
            .unwrap_or_default()
}

fn remove_extension(path: &mut PathBuf) {
    if let Some(stem) = path
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .map(ToOwned::to_owned)
    {
        path.pop();
        path.push(stem);
    }
}
