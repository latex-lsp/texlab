use crate::completion::factory;
use crate::completion::latex::combinators::LatexCombinators;
use crate::feature::FeatureRequest;
use crate::syntax::latex::LatexCommand;
use lsp_types::{CompletionItem, CompletionParams};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use walkdir::WalkDir;

const NO_EXTENSION_COMMANDS: &'static [&'static str] = &["\\include", "\\includesvg"];

const COMMAND_NAMES: &'static [&'static str] = &[
    "\\include",
    "\\input",
    "\\bibliography",
    "\\addbibresource",
    "\\includegraphics",
    "\\includesvg",
];

pub struct LatexIncludeCompletionProvider;

impl LatexIncludeCompletionProvider {
    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        await!(LatexCombinators::argument(
            request,
            COMMAND_NAMES,
            0,
            async move |command| {
                if request.document.uri.scheme() != "file" {
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
                        if entry.file_type().is_file() && is_included(command, &entry.path()) {
                            let mut path = entry.into_path();
                            if NO_EXTENSION_COMMANDS.contains(&command.name.text()) {
                                remove_extension(&mut path);
                            }
                            items.push(factory::create_file(&path));
                        } else if entry.file_type().is_dir() {
                            let path = entry.into_path();
                            items.push(factory::create_folder(&path));
                        }
                    }
                }
                items
            }
        ))
    }
}

fn current_directory(
    request: &FeatureRequest<CompletionParams>,
    command: &LatexCommand,
) -> PathBuf {
    let mut path = PathBuf::from_str(request.document.uri.path()).unwrap();
    path.pop();
    if let Some(include) = command.extract_word(0) {
        path.push(include.text());
        if !include.text().ends_with("/") {
            path.pop();
        }
    }
    path
}

fn is_included(command: &LatexCommand, file: &Path) -> bool {
    let allowed_extensions = allowed_extensions(command);
    file.extension()
        .map(|extension| extension.to_string_lossy().to_lowercase())
        .map(|extension| allowed_extensions.contains(&extension.as_str()))
        .unwrap_or(false)
}

fn allowed_extensions(command: &LatexCommand) -> Vec<&'static str> {
    match command.name.text() {
        "\\include" | "\\input" => vec!["tex"],
        "\\bibliography" | "\\addbibresource" => vec!["bib"],
        "\\includegraphics" => vec!["pdf", "png", "jpg", "jpeg", "bmp"],
        "\\includesvg" => vec!["svg"],
        _ => vec![],
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
