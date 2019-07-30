use crate::completion::factory;
use crate::completion::latex::combinators::{self, Parameter};
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams, Range, TextEdit};
use std::path::{Path, PathBuf};
use texlab_syntax::*;
use texlab_workspace::*;
use walkdir::WalkDir;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexIncludeCompletionProvider;

impl FeatureProvider for LatexIncludeCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let parameters = LANGUAGE_DATA
            .include_commands
            .iter()
            .map(|cmd| Parameter::new(&cmd.name, cmd.index));

        combinators::argument_word(request, parameters, async move |command, index| {
            if !request.document().is_file() {
                return Vec::new();
            }

            let position = request.params.position;
            let mut items = Vec::new();
            let path_word = command.extract_word(index);
            let name_range = match path_word {
                Some(path_word) => Range::new_simple(
                    path_word.start().line,
                    path_word.end().character
                        - path_word.text().split('/').last().unwrap().chars().count() as u64,
                    path_word.end().line,
                    path_word.end().character,
                ),
                None => Range::new(position, position),
            };
            let directory = current_directory(&request, &command);

            for entry in WalkDir::new(directory)
                .min_depth(1)
                .max_depth(1)
                .follow_links(false)
                .into_iter()
                .filter_map(std::result::Result::ok)
            {
                if entry.file_type().is_file() && is_included(&command, &entry.path()) {
                    let mut path = entry.into_path();
                    let include_extension = LANGUAGE_DATA
                        .include_commands
                        .iter()
                        .find(|cmd| command.name.text() == cmd.name)
                        .unwrap()
                        .include_extension;

                    if !include_extension {
                        remove_extension(&mut path);
                    }
                    let text_edit = make_text_edit(name_range, &path);
                    items.push(factory::file(request, &path, text_edit));
                } else if entry.file_type().is_dir() {
                    let path = entry.into_path();
                    let text_edit = make_text_edit(name_range, &path);
                    items.push(factory::folder(request, &path, text_edit));
                }
            }
            items
        })
        .await
    }
}

fn current_directory(
    request: &FeatureRequest<CompletionParams>,
    command: &LatexCommand,
) -> PathBuf {
    let mut path = request.document().uri.to_file_path().unwrap();
    path = PathBuf::from(path.to_string_lossy().into_owned().replace('\\', "/"));

    path.pop();
    if let Some(include) = command.extract_word(0) {
        path.push(include.text());
        if !include.text().ends_with('/') {
            path.pop();
        }
    }
    path
}

fn is_included(command: &LatexCommand, file: &Path) -> bool {
    if let Some(allowed_extensions) = LANGUAGE_DATA
        .include_commands
        .iter()
        .find(|cmd| command.name.text() == cmd.name)
        .unwrap()
        .kind
        .extensions()
    {
        file.extension()
            .map(|extension| extension.to_string_lossy().to_lowercase())
            .map(|extension| allowed_extensions.contains(&extension.as_str()))
            .unwrap_or(false)
    } else {
        true
    }
}

fn remove_extension(path: &mut PathBuf) {
    let stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned());

    if let Some(stem) = stem {
        path.pop();
        path.push(PathBuf::from(stem));
    }
}

fn make_text_edit(range: Range, path: &Path) -> TextEdit {
    let text = path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned()
        .into();
    TextEdit::new(range, text)
}
