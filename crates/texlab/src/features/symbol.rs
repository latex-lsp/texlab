mod bibtex;
mod latex;
mod project_order;
mod types;

use std::cmp::Reverse;

use base_db::Workspace;
use lsp_types::{
    ClientCapabilities, DocumentSymbolResponse, SymbolInformation, Url, WorkspaceSymbolParams,
};

use crate::util::capabilities::ClientCapabilitiesExt;

use self::{project_order::ProjectOrdering, types::InternalSymbol};

pub fn find_document_symbols(
    workspace: &Workspace,
    uri: &Url,
    client_capabilties: &ClientCapabilities,
) -> Option<DocumentSymbolResponse> {
    let document = workspace.lookup(uri)?;
    let related = workspace.related(document);

    let mut buf = Vec::new();
    latex::find_symbols(workspace, &related, document, &mut buf);
    bibtex::find_symbols(document, &mut buf);

    let config = &workspace.config().symbols;

    InternalSymbol::filter(&mut buf, config);

    if client_capabilties.has_hierarchical_document_symbol_support() {
        let symbols = buf
            .into_iter()
            .map(|symbol| symbol.into_document_symbol())
            .collect();

        Some(DocumentSymbolResponse::Nested(symbols))
    } else {
        let mut new_buf = Vec::new();
        for symbol in buf {
            symbol.flatten(&mut new_buf);
        }

        let mut new_buf: Vec<_> = new_buf
            .into_iter()
            .map(|symbol| symbol.into_symbol_info(uri.clone()))
            .collect();

        sort_symbols(workspace, &mut new_buf);
        Some(DocumentSymbolResponse::Flat(new_buf))
    }
}

#[derive(Debug, Clone)]
struct WorkspaceSymbol {
    info: SymbolInformation,
    search_text: String,
}

#[must_use]
pub fn find_workspace_symbols(
    workspace: &Workspace,
    params: &WorkspaceSymbolParams,
) -> Vec<SymbolInformation> {
    let mut symbols = Vec::new();

    for document in workspace.iter() {
        let related = workspace.related(document);

        let mut buf = Vec::new();
        latex::find_symbols(workspace, &related, document, &mut buf);
        bibtex::find_symbols(document, &mut buf);
        let mut new_buf = Vec::new();

        for symbol in buf {
            symbol.flatten(&mut new_buf);
        }

        for symbol in new_buf {
            symbols.push(WorkspaceSymbol {
                search_text: symbol.search_text(),
                info: symbol.into_symbol_info(document.uri.clone()),
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
    let ordering = ProjectOrdering::new(workspace);
    symbols.sort_by(|left, right| {
        let left_key = (
            ordering.get(&left.location.uri),
            left.location.range.start,
            Reverse(left.location.range.end),
        );
        let right_key = (
            ordering.get(&right.location.uri),
            right.location.range.start,
            Reverse(right.location.range.end),
        );
        left_key.cmp(&right_key)
    });
}
