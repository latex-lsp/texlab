use crate::completion::factory;
use crate::completion::latex::combinators::{self, ArgumentLocation};
use crate::data::language::{language_data, LatexIncludeKind};
use crate::feature::{FeatureProvider, FeatureRequest};
use crate::syntax::latex::LatexCommand;
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::WalkDir;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexIncludeCompletionProvider;

impl FeatureProvider for LatexIncludeCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<Arc<CompletionItem>>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let locations = language_data()
            .include_commands
            .iter()
            .filter(|cmd| match cmd.kind {
                LatexIncludeKind::Package | LatexIncludeKind::Class => false,
                LatexIncludeKind::Latex
                | LatexIncludeKind::Bibliography
                | LatexIncludeKind::Image
                | LatexIncludeKind::Svg
                | LatexIncludeKind::Pdf
                | LatexIncludeKind::Everything => true,
            })
            .map(|cmd| ArgumentLocation::new(&cmd.name, cmd.index));

        combinators::argument(request, locations, async move |command| {
            if !request.document().is_file() {
                return Vec::new();
            }

            let mut items = Vec::new();
            let directory = current_directory(&request, &command);
            for entry in WalkDir::new(directory)
                .min_depth(1)
                .max_depth(1)
                .follow_links(false)
                .into_iter()
            {
                if let Ok(entry) = entry {
                    if entry.file_type().is_file() && is_included(request, &command, &entry.path())
                    {
                        let mut path = entry.into_path();
                        let include_extension = language_data()
                            .include_commands
                            .iter()
                            .find(|cmd| command.name.text() == cmd.name)
                            .unwrap()
                            .include_extension;

                        if !include_extension {
                            remove_extension(&mut path);
                        }
                        items.push(Arc::new(factory::create_file(&path)));
                    } else if entry.file_type().is_dir() {
                        let path = entry.into_path();
                        items.push(Arc::new(factory::create_folder(&path)));
                    }
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

fn is_included(
    request: &FeatureRequest<CompletionParams>,
    command: &LatexCommand,
    file: &Path,
) -> bool {
    if let Some(allowed_extensions) = language_data()
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
