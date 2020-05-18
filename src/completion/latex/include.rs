use super::combinators::{self, Parameter};
use crate::{
    completion::types::{Item, ItemData},
    feature::FeatureRequest,
    protocol::{CompletionParams, Range, RangeExt},
    syntax::{latex, AstNodeIndex, SyntaxNode, LANGUAGE_DATA},
};
use std::path::{Path, PathBuf};
use tokio::fs;

pub async fn complete_latex_includes<'a>(
    req: &'a FeatureRequest<CompletionParams>,
    items: &mut Vec<Item<'a>>,
) {
    let parameters = LANGUAGE_DATA.include_commands.iter().map(|cmd| Parameter {
        name: &cmd.name[1..],
        index: cmd.index,
    });

    combinators::argument_word(req, parameters, |cmd_node, index| async move {
        if !req.current().is_file() {
            return;
        }

        make_items(req, items, cmd_node, index).await;
    })
    .await;
}

async fn make_items<'a>(
    req: &'a FeatureRequest<CompletionParams>,
    items: &mut Vec<Item<'a>>,
    cmd_node: AstNodeIndex,
    index: usize,
) -> Option<()> {
    let table = req.current().content.as_latex()?;
    let pos = req.params.text_document_position.position;
    let path_word = table.extract_word(cmd_node, latex::GroupKind::Group, index);
    let name_range = match path_word {
        Some(path_word) => Range::new_simple(
            path_word.start().line,
            path_word.end().character - path_word.text().split('/').last()?.chars().count() as u64,
            path_word.end().line,
            path_word.end().character,
        ),
        None => Range::new(pos, pos),
    };

    let cmd = table.as_command(cmd_node)?;
    let current_dir = current_dir(req, table, cmd_node)?;
    let mut entries = fs::read_dir(current_dir).await.ok()?;
    while let Some(entry) = entries.next_entry().await.ok()? {
        let mut path = entry.path();

        let file_type = entry.file_type().await.ok()?;
        if file_type.is_file() && is_included(&cmd, &path) {
            let include_extension = LANGUAGE_DATA
                .include_commands
                .iter()
                .find(|c| cmd.name.text() == c.name)?
                .include_extension;

            if !include_extension {
                remove_extension(&mut path);
            }
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let item = Item::new(name_range, ItemData::File { name });
            items.push(item);
        } else if file_type.is_dir() {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let item = Item::new(name_range, ItemData::Directory { name });
            items.push(item);
        }
    }
    Some(())
}

fn current_dir(
    req: &FeatureRequest<CompletionParams>,
    table: &latex::SymbolTable,
    cmd_node: AstNodeIndex,
) -> Option<PathBuf> {
    let mut path = req
        .options
        .latex
        .as_ref()
        .and_then(|latex| latex.root_directory.as_ref())
        .map_or_else(
            || {
                let mut path = req.current().uri.to_file_path().unwrap();
                path.pop();
                path
            },
            Clone::clone,
        );

    path = PathBuf::from(path.to_str()?.replace('\\', "/"));
    if let Some(include) = table.extract_word(cmd_node, latex::GroupKind::Group, 0) {
        path.push(include.text());
        if !include.text().ends_with('/') {
            path.pop();
        }
    }
    Some(path)
}

fn is_included(cmd: &latex::Command, file: &Path) -> bool {
    if let Some(allowed_extensions) = LANGUAGE_DATA
        .include_commands
        .iter()
        .find(|c| c.name == cmd.name.text())
        .and_then(|c| c.kind.extensions())
    {
        file.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
            .map(|ext| allowed_extensions.contains(&ext.as_str()))
            .unwrap_or_default()
    } else {
        true
    }
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
