use super::combinators::{self, Parameter};
use crate::{
    completion::factory,
    feature::{FeatureProvider, FeatureRequest},
    protocol::{CompletionItem, CompletionParams, Range, RangeExt, TextEdit},
    syntax::{latex, SyntaxNode, LANGUAGE_DATA},
};
use futures_boxed::boxed;
use petgraph::graph::NodeIndex;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexIncludeCompletionProvider;

impl FeatureProvider for LatexIncludeCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let parameters = LANGUAGE_DATA.include_commands.iter().map(|cmd| Parameter {
            name: &cmd.name,
            index: cmd.index,
        });

        combinators::argument_word(req, parameters, |cmd_node, index| async move {
            log::info!("Include!");
            if !req.current().is_file() {
                return Vec::new();
            }

            Self::make_items(req, cmd_node, index)
                .await
                .unwrap_or_default()
        })
        .await
    }
}

impl LatexIncludeCompletionProvider {
    async fn make_items(
        req: &FeatureRequest<CompletionParams>,
        cmd_node: NodeIndex,
        index: usize,
    ) -> Option<Vec<CompletionItem>> {
        let table = req.current().content.as_latex()?;
        let pos = req.params.text_document_position.position;
        let mut items = Vec::new();
        let path_word = table
            .tree
            .extract_word(cmd_node, latex::GroupKind::Group, index);
        let name_range = match path_word {
            Some(path_word) => Range::new_simple(
                path_word.start().line,
                path_word.end().character
                    - path_word.text().split('/').last()?.chars().count() as u64,
                path_word.end().line,
                path_word.end().character,
            ),
            None => Range::new(pos, pos),
        };

        let cmd = table.tree.as_command(cmd_node)?;
        let current_dir = Self::current_dir(req, table, cmd_node)?;
        log::info!("Current Dir = {:?}", current_dir);
        let mut entries = fs::read_dir(current_dir).await.ok()?;
        while let Some(entry) = entries.next_entry().await.ok()? {
            let mut path = entry.path();
            let file_type = entry.file_type().await.ok()?;
            if file_type.is_file() && Self::is_included(&cmd, &path) {
                let include_extension = LANGUAGE_DATA
                    .include_commands
                    .iter()
                    .find(|c| cmd.name.text() == c.name)?
                    .include_extension;

                if !include_extension {
                    Self::remove_extension(&mut path);
                }
                let text_edit = Self::make_text_edit(name_range, &path)?;
                items.push(factory::file(req, &path, text_edit));
            } else if file_type.is_dir() {
                let text_edit = Self::make_text_edit(name_range, &path)?;
                items.push(factory::folder(req, &path, text_edit));
            }
        }
        Some(items)
    }

    fn current_dir(
        req: &FeatureRequest<CompletionParams>,
        table: &latex::SymbolTable,
        cmd_node: NodeIndex,
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
        if let Some(include) = table
            .tree
            .extract_word(cmd_node, latex::GroupKind::Group, 0)
        {
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

    fn make_text_edit(range: Range, path: &Path) -> Option<TextEdit> {
        let text = path.file_name()?.to_str()?.into();
        let edit = TextEdit::new(range, text);
        Some(edit)
    }
}
