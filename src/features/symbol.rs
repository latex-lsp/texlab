mod bibtex;
mod latex;
mod project_order;
mod types;

use std::{cmp::Reverse, sync::Arc};

use cancellation::CancellationToken;
use lsp_types::{
    DocumentSymbolParams, DocumentSymbolResponse, SymbolInformation, WorkspaceSymbolParams,
};

use crate::{ClientCapabilitiesExt, Uri, Workspace};

use self::{
    bibtex::find_bibtex_symbols, latex::find_latex_symbols, project_order::ProjectOrdering,
};

use super::FeatureRequest;

pub fn find_document_symbols(
    req: FeatureRequest<DocumentSymbolParams>,
    token: &CancellationToken,
) -> DocumentSymbolResponse {
    let mut buf = Vec::new();
    find_latex_symbols(&req.subset, &mut buf, token);
    find_bibtex_symbols(&req.subset, &mut buf, token);
    if req
        .context
        .client_capabilities
        .lock()
        .unwrap()
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
        let mut new_buf = new_buf
            .into_iter()
            .map(|symbol| symbol.into_symbol_info(req.main_document().uri.as_ref().clone()))
            .collect();
        sort_symbols(req.workspace.as_ref(), &mut new_buf);
        DocumentSymbolResponse::Flat(new_buf)
    }
}

#[derive(Debug, Clone)]
struct WorkspaceSymbol {
    info: SymbolInformation,
    search_text: String,
}

pub fn find_workspace_symbols(
    workspace: &dyn Workspace,
    params: &WorkspaceSymbolParams,
    token: &CancellationToken,
) -> Vec<SymbolInformation> {
    let mut symbols = Vec::new();

    for document in workspace.documents() {
        if let Some(subset) = workspace.subset(Arc::clone(&document.uri)) {
            let mut buf = Vec::new();
            find_latex_symbols(&subset, &mut buf, token);
            find_bibtex_symbols(&subset, &mut buf, token);
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

fn sort_symbols(workspace: &dyn Workspace, symbols: &mut Vec<SymbolInformation>) {
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
