use std::{
    convert::TryFrom,
    fs,
    path::{Path, PathBuf},
};

use lsp_types::CompletionParams;
use rowan::{ast::AstNode, TextRange, TextSize};

use crate::{features::cursor::CursorContext, syntax::latex};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_includes<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
) -> Option<()> {
    if context.request.main_document().uri.scheme() != "file" {
        return None;
    }

    let (path_text, path_range, group) = context.find_curly_group_word_list()?;

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

    let segment_range = if path_text.is_empty() {
        path_range
    } else {
        let start =
            path_range.end() - TextSize::try_from(path_text.split('/').last()?.len()).ok()?;
        TextRange::new(start, path_range.end())
    };

    let mut dirs = vec![current_dir(context, &path_text, None)];
    if include.kind() == latex::GRAPHICS_INCLUDE {
        for document in context.request.workspace.documents_by_uri.values() {
            if let Some(data) = document.data.as_latex() {
                for graphics_path in &data.extras.graphics_paths {
                    dirs.push(current_dir(context, &path_text, Some(graphics_path)));
                }
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
            let name = path.file_name()?.to_str()?.into();
            let data = InternalCompletionItemData::File { name };
            let item = InternalCompletionItem::new(segment_range, data);
            items.push(item);
        } else if file_type.is_dir() {
            let name = path.file_name()?.to_str()?.into();
            let data = InternalCompletionItemData::Directory { name };
            let item = InternalCompletionItem::new(segment_range, data);
            items.push(item);
        }
    }

    Some(())
}

fn current_dir(
    context: &CursorContext<CompletionParams>,
    path_text: &str,
    graphics_path: Option<&str>,
) -> Option<PathBuf> {
    let mut path = context
        .request
        .workspace
        .options
        .root_directory
        .as_ref()
        .map(|root_directory| {
            context
                .request
                .workspace
                .current_directory
                .join(root_directory)
        })
        .unwrap_or_else(|| {
            let mut path = context.request.main_document().uri.to_file_path().unwrap();
            path.pop();
            path
        });

    path = PathBuf::from(path.to_str()?.replace('\\', "/"));
    if !path_text.is_empty() {
        if let Some(graphics_path) = graphics_path {
            path.push(graphics_path);
        }

        path.push(&path_text);
        if !path_text.ends_with('/') {
            path.pop();
        }
    }
    Some(path)
}

fn is_included(file: &Path, allowed_extensions: &[&str]) -> bool {
    allowed_extensions.is_empty()
        || file
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
            .map(|ext| allowed_extensions.contains(&ext.as_str()))
            .unwrap_or_default()
}

fn remove_extension(path: &mut PathBuf) {
    if let Some(stem) = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(ToOwned::to_owned)
    {
        path.pop();
        path.push(stem);
    }
}

#[cfg(test)]
mod tests {
    use crate::features::testing::FeatureTester;

    use super::*;

    #[test]
    fn test_empty_latex_document() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "")])
            .main("main.tex")
            .line(0)
            .character(0)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_includes(&context, &mut actual_items);

        assert!(actual_items.is_empty());
    }

    #[test]
    fn test_empty_bibtex_document() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "")])
            .main("main.bib")
            .line(0)
            .character(0)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_includes(&context, &mut actual_items);

        assert!(actual_items.is_empty());
    }
}
