use base_db::{Document, Workspace};
use lsp_types::{DocumentSymbolResponse, WorkspaceSymbolResponse};

use crate::util::{to_proto, ClientFlags};

pub fn document_symbols(
    workspace: &Workspace,
    document: &Document,
    client_flags: &ClientFlags,
) -> DocumentSymbolResponse {
    let symbols = symbols::document_symbols(workspace, document);
    if client_flags.hierarchical_document_symbols {
        let results = symbols
            .into_iter()
            .filter_map(|symbol| to_proto::document_symbol(symbol, &document.line_index))
            .collect();

        DocumentSymbolResponse::Nested(results)
    } else {
        let mut results = Vec::new();
        for symbol in symbols {
            to_proto::symbol_information(symbol, document, &mut results);
        }

        DocumentSymbolResponse::Flat(results)
    }
}

pub fn workspace_symbols(workspace: &Workspace, query: &str) -> WorkspaceSymbolResponse {
    let symbols = symbols::workspace_symbols(workspace, query);
    let mut results = Vec::new();
    for symbols::SymbolLocation { symbol, document } in symbols {
        to_proto::symbol_information(symbol, document, &mut results);
    }

    WorkspaceSymbolResponse::Flat(results)
}
