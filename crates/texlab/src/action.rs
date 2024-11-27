use std::collections::HashMap;

use base_db::Workspace;
use diagnostics::{Diagnostic, ImportError};
use lsp_types::{CodeAction, CodeActionKind, CodeActionParams, TextEdit, WorkspaceEdit};
use rowan::{TextRange, TextSize};
use rustc_hash::FxBuildHasher;

use crate::util::{line_index_ext::LineIndexExt, to_proto};

pub fn remove_duplicate_imports(
    diagnostics_map: HashMap<lsp_types::Url, Vec<diagnostics::Diagnostic>, FxBuildHasher>,
    params: CodeActionParams,
    workspace: parking_lot::lock_api::RwLockReadGuard<'_, parking_lot::RawRwLock, Workspace>,
) -> Vec<CodeAction> {
    let mut actions = Vec::new();
    let url = params.text_document.uri.clone();
    let document = workspace.lookup(&url).unwrap();

    let diagnostics = diagnostics_map
        .get(&url)
        .unwrap()
        .iter()
        .filter(|diag| matches!(diag, Diagnostic::Import(_, ImportError::DuplicateImport(_))))
        .collect::<Vec<_>>();

    let cursor_position = params.range.start;

    let cursor_diag = diagnostics.iter().find(|diag| {
        let line_index = &workspace
            .lookup(&params.text_document.uri)
            .unwrap()
            .line_index;
        let range = diag.range(&line_index).unwrap();
        range.contains(line_index.offset_lsp(cursor_position).unwrap())
    });

    if let Some(diag) = cursor_diag {
        let package_name = get_package_name(&workspace, &params, diag);

        let filtered_diagnostics: Vec<_> = diagnostics.clone()
            .into_iter()
            .filter(|diag| get_package_name(&workspace, &params, diag) == package_name)
            .collect();

        let import_diags: Vec<lsp_types::Diagnostic> = filtered_diagnostics.clone()
        .iter()
        .map(|diag| to_proto::diagnostic(&workspace, document, diag).unwrap())
        .collect();

        for diag in filtered_diagnostics {
            let line_index = &workspace
                .lookup(&params.text_document.uri)
                .unwrap()
                .line_index;
            let range = diag.range(&line_index).unwrap();
            let start_line = line_index
                .line_col_lsp_range(TextRange::at(range.start(), TextSize::from(0)))
                .unwrap()
                .start
                .line;

            actions.push(CodeAction {
                title: format!(
                    "Remove duplicate package: {}, line {}.",
                    get_package_name(&workspace, &params, diag),
                    (1 + start_line)
                ),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: Some(import_diags.clone()),
                edit: Some(WorkspaceEdit {
                    changes: Some(
                        vec![(
                            url.clone(),
                            vec![TextEdit {
                                range: get_line_range(&workspace, &params, diag),
                                new_text: "".to_string(),
                            }],
                        )]
                        .into_iter()
                        .collect(),
                    ),
                    document_changes: None,
                    change_annotations: None,
                }),
                command: None,
                is_preferred: None,
                disabled: None,
                data: None,
            });
        }
    }

    let mut seen_packages = HashMap::new();
    let all_diags = diagnostics
        .iter()
        .filter(|diag| {
            let package_name = get_package_name(&workspace, &params, diag);
            if seen_packages.contains_key(&package_name) {
                true
            } else {
                seen_packages.insert(package_name, true);
                false
            }
        })
        .collect::<Vec<_>>();
        
    let mut all_diags_edit = Vec::new();


    for diag in all_diags {
        all_diags_edit.push(TextEdit {
            range: get_line_range(&workspace, &params, diag),
            new_text: "".to_string(),
        });
        
    }    
    
    if all_diags_edit.is_empty() {
        return actions;
    }

    let import_diags: Vec<lsp_types::Diagnostic> = diagnostics.clone()
        .iter()
        .map(|diag| to_proto::diagnostic(&workspace, document, diag).unwrap())
        .collect();



    actions.push(CodeAction {
        title: "Remove all duplicate package.".to_string(),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(import_diags.clone()),
        edit: Some(WorkspaceEdit {
            changes: Some(
                vec![(
                    url.clone(),
                    all_diags_edit,
                )]
                .into_iter()
                .collect(),
            ),
            document_changes: None,
            change_annotations: None,
        }),
        command: None,
        is_preferred: None,
        disabled: None,
        data: None,
    });

    actions
}

fn get_line_range(
    workspace: &Workspace,
    params: &CodeActionParams,
    diag: &Diagnostic,
) -> lsp_types::Range {
    let line_index = &workspace
        .lookup(&params.text_document.uri)
        .unwrap()
        .line_index;
    let range = diag.range(&line_index).unwrap();
    let start_line = line_index
        .line_col_lsp_range(TextRange::at(range.start(), TextSize::from(0)))
        .unwrap()
        .start
        .line;
    let end_line = line_index
        .line_col_lsp_range(TextRange::new(range.end(), range.end()))
        .unwrap()
        .end
        .line;
    let start = lsp_types::Position::new(start_line, 0);
    let end = lsp_types::Position::new(end_line + 1, 0);
    lsp_types::Range::new(start, end)
}

fn get_package_name(workspace: &Workspace, params: &CodeActionParams, diag: &Diagnostic) -> String {
    let document = workspace.lookup(&params.text_document.uri).unwrap();
    let line_index = document.line_index.clone();
    let range = diag.range(&line_index).unwrap();
    let start = usize::from(range.start());
    let end = usize::from(range.end());
    let text = &document.text[start..end];
    text.to_string()
}
