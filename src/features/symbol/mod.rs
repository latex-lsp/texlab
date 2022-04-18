mod bibtex;
mod latex;
mod project_order;
mod types;

use std::{cmp::Reverse, sync::Arc};

use lsp_types::{
    DocumentSymbolParams, DocumentSymbolResponse, PartialResultParams, SymbolInformation,
    TextDocumentIdentifier, WorkDoneProgressParams, WorkspaceSymbolParams,
};

use crate::{ClientCapabilitiesExt, Uri, Workspace};

use self::{
    bibtex::find_bibtex_symbols, latex::find_latex_symbols, project_order::ProjectOrdering,
};

use super::FeatureRequest;

pub fn find_document_symbols(req: FeatureRequest<DocumentSymbolParams>) -> DocumentSymbolResponse {
    let mut buf = Vec::new();
    find_latex_symbols(&req, &mut buf);
    find_bibtex_symbols(&req, &mut buf);
    if req
        .workspace
        .client_capabilities
        .has_hierarchical_document_symbol_support()
    {
        DocumentSymbolResponse::Nested(
            buf.into_iter()
                .map(|symbol| symbol.into_document_symbol())
                .collect(),
        )
    } else {
        let mut new_buf = Vec::new();
        for symbol in buf {
            symbol.flatten(&mut new_buf);
        }
        let mut new_buf: Vec<_> = new_buf
            .into_iter()
            .map(|symbol| symbol.into_symbol_info(req.main_document().uri.as_ref().clone()))
            .collect();
        sort_symbols(&req.workspace, &mut new_buf);
        DocumentSymbolResponse::Flat(new_buf)
    }
}

#[derive(Debug, Clone)]
struct WorkspaceSymbol {
    info: SymbolInformation,
    search_text: String,
}

pub fn find_workspace_symbols(
    workspace: &Workspace,
    params: &WorkspaceSymbolParams,
) -> Vec<SymbolInformation> {
    let mut symbols = Vec::new();

    for document in workspace.documents_by_uri.values() {
        let request = FeatureRequest {
            uri: Arc::clone(&document.uri),
            params: DocumentSymbolParams {
                text_document: TextDocumentIdentifier::new(document.uri.as_ref().clone().into()),
                partial_result_params: PartialResultParams::default(),
                work_done_progress_params: WorkDoneProgressParams::default(),
            },
            workspace: workspace.slice(&document.uri),
        };

        let mut buf = Vec::new();
        find_latex_symbols(&request, &mut buf);
        find_bibtex_symbols(&request, &mut buf);
        let mut new_buf = Vec::new();

        for symbol in buf {
            symbol.flatten(&mut new_buf);
        }

        for symbol in new_buf {
            symbols.push(WorkspaceSymbol {
                search_text: symbol.search_text(),
                info: symbol.into_symbol_info(document.uri.as_ref().clone()),
            });
        }
    }

    let query_words: Vec<_> = params
        .query
        .split_whitespace()
        .map(str::to_lowercase)
        .collect();
    let mut filtered = Vec::new();
    for symbol in symbols {
        let mut included = true;
        for word in &query_words {
            if !symbol.search_text.contains(word) {
                included = false;
                break;
            }
        }

        if included {
            filtered.push(symbol.info);
        }
    }
    sort_symbols(workspace, &mut filtered);
    filtered
}

fn sort_symbols(workspace: &Workspace, symbols: &mut [SymbolInformation]) {
    let ordering = ProjectOrdering::from(workspace);
    symbols.sort_by(|left, right| {
        let left_key = (
            ordering.get(&Uri::from(left.location.uri.clone())),
            left.location.range.start,
            Reverse(left.location.range.end),
        );
        let right_key = (
            ordering.get(&Uri::from(right.location.uri.clone())),
            right.location.range.start,
            Reverse(right.location.range.end),
        );
        left_key.cmp(&right_key)
    });
}
