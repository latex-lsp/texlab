use base_db::{Document, Workspace};
use rowan::{TextLen, TextRange, TextSize};
use rustc_hash::FxHashMap;
use syntax::BuildError;
use url::Url;

use crate::{Diagnostic, DiagnosticSource, ErrorCode};

#[derive(Debug, Default)]
struct BuildLog {
    errors: FxHashMap<Url, Vec<Diagnostic>>,
}

#[derive(Debug, Default)]
pub struct BuildErrors {
    logs: FxHashMap<Url, BuildLog>,
}

impl DiagnosticSource for BuildErrors {
    fn on_change(&mut self, workspace: &Workspace, log_document: &Document) {
        let mut errors: FxHashMap<Url, Vec<Diagnostic>> = FxHashMap::default();

        let Some(data) = log_document.data.as_log() else { return };

        let parents = workspace.parents(log_document);
        let Some(root_document) = parents.iter().next() else { return };

        let Some(base_path) = root_document
            .path
            .as_deref()
            .and_then(|path| path.parent()) else { return };

        for error in &data.errors {
            let full_path = base_path.join(&error.relative_path);
            let Ok(full_path_uri) = Url::from_file_path(&full_path) else { continue };
            let tex_document = workspace.lookup(&full_path_uri).unwrap_or(root_document);

            let range = find_range_of_hint(tex_document, error).unwrap_or_else(|| {
                let line = error.line.unwrap_or(0);
                let offset = *tex_document
                    .line_index
                    .newlines
                    .get(line as usize)
                    .unwrap_or(&TextSize::from(0));

                TextRange::empty(offset)
            });

            let diagnostic = Diagnostic {
                range,
                code: ErrorCode::Build(error.clone()),
            };

            errors
                .entry(tex_document.uri.clone())
                .or_default()
                .push(diagnostic);
        }

        self.logs
            .insert(log_document.uri.clone(), BuildLog { errors });
    }

    fn cleanup(&mut self, workspace: &Workspace) {
        self.logs.retain(|uri, _| workspace.lookup(uri).is_some());
    }

    fn publish<'this, 'db>(
        &'this mut self,
        workspace: &'db Workspace,
        results: &mut FxHashMap<&'db Url, Vec<&'this Diagnostic>>,
    ) {
        for document in workspace.iter() {
            let Some(log) = self.logs.get(&document.uri) else { continue };
            for (uri, errors) in &log.errors {
                let Some(uri) = workspace.lookup(uri).map(|doc| &doc.uri) else { continue };
                results.entry(uri).or_default().extend(errors.iter());
            }
        }
    }
}

fn find_range_of_hint(document: &Document, error: &BuildError) -> Option<TextRange> {
    let line = error.line? as usize;
    let hint = error.hint.as_deref()?;
    let line_index = &document.line_index;

    let line_start = line_index.newlines.get(line).copied()?;
    let line_end = line_index
        .newlines
        .get(line + 1)
        .copied()
        .unwrap_or((&document.text).text_len());

    let line_text = &document.text[line_start.into()..line_end.into()];
    let hint_start = line_start + TextSize::try_from(line_text.find(hint)?).unwrap();
    let hint_end = hint_start + hint.text_len();
    Some(TextRange::new(hint_start, hint_end))
}
