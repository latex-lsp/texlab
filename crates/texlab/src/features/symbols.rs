use base_db::Workspace;

use crate::util::{from_proto, to_proto, ClientFlags};

pub fn document_symbols(
    workspace: &Workspace,
    params: lsp_types::DocumentSymbolParams,
    client_flags: &ClientFlags,
) -> Option<lsp_types::DocumentSymbolResponse> {
    let params = from_proto::feature_params(workspace, params.text_document)?;
    let symbols = symbols::document_symbols(workspace, params.document);
    Some(to_proto::document_symbol_response(
        params.document,
        symbols,
        client_flags,
    ))
}

pub fn workspace_symbols(workspace: &Workspace, query: &str) -> lsp_types::WorkspaceSymbolResponse {
    let symbols = symbols::workspace_symbols(workspace, query);
    let mut results = Vec::new();
    for symbols::SymbolLocation { symbol, document } in symbols {
        to_proto::symbol_information(symbol, document, &mut results);
    }

    lsp_types::WorkspaceSymbolResponse::Flat(results)
}
