use base_db::{data::BibtexEntryTypeCategory, Document, Workspace};
use lsp_types::{
    ClientCapabilities, DocumentSymbol, DocumentSymbolResponse, Location, WorkspaceSymbolResponse,
};

use crate::util::{capabilities::ClientCapabilitiesExt, line_index_ext::LineIndexExt};

pub fn document_symbols(
    workspace: &Workspace,
    document: &Document,
    capabilities: &ClientCapabilities,
) -> DocumentSymbolResponse {
    let symbols = symbols::document_symbols(workspace, document);
    if capabilities.has_hierarchical_document_symbol_support() {
        let results = symbols
            .into_iter()
            .map(|symbol| convert_to_nested_symbol(symbol, document))
            .collect();

        DocumentSymbolResponse::Nested(results)
    } else {
        let mut results = Vec::new();
        for symbol in symbols {
            convert_to_flat_symbols(symbol, document, &mut results);
        }

        DocumentSymbolResponse::Flat(results)
    }
}

pub fn workspace_symbols(workspace: &Workspace, query: &str) -> WorkspaceSymbolResponse {
    let symbols = symbols::workspace_symbols(workspace, query);
    let mut results = Vec::new();
    for symbols::SymbolLocation { symbol, document } in symbols {
        convert_to_flat_symbols(symbol, document, &mut results);
    }

    WorkspaceSymbolResponse::Flat(results)
}

fn convert_to_nested_symbol(symbol: symbols::Symbol, document: &Document) -> DocumentSymbol {
    let children = symbol
        .children
        .into_iter()
        .map(|child| convert_to_nested_symbol(child, document))
        .collect();

    #[allow(deprecated)]
    DocumentSymbol {
        name: symbol.name,
        detail: symbol.label.map(|label| label.text),
        kind: convert_symbol_kind(symbol.kind),
        deprecated: Some(false),
        range: document.line_index.line_col_lsp_range(symbol.full_range),
        selection_range: document
            .line_index
            .line_col_lsp_range(symbol.selection_range),
        children: Some(children),
        tags: None,
    }
}

fn convert_to_flat_symbols(
    symbol: symbols::Symbol,
    document: &Document,
    results: &mut Vec<lsp_types::SymbolInformation>,
) {
    let range = document.line_index.line_col_lsp_range(symbol.full_range);

    #[allow(deprecated)]
    results.push(lsp_types::SymbolInformation {
        name: symbol.name,
        kind: convert_symbol_kind(symbol.kind),
        deprecated: Some(false),
        location: Location::new(document.uri.clone(), range),
        tags: None,
        container_name: None,
    });

    for child in symbol.children {
        convert_to_flat_symbols(child, document, results);
    }
}

fn convert_symbol_kind(value: symbols::SymbolKind) -> lsp_types::SymbolKind {
    match value {
        symbols::SymbolKind::Section => lsp_types::SymbolKind::MODULE,
        symbols::SymbolKind::Figure => lsp_types::SymbolKind::METHOD,
        symbols::SymbolKind::Algorithm => lsp_types::SymbolKind::METHOD,
        symbols::SymbolKind::Table => lsp_types::SymbolKind::METHOD,
        symbols::SymbolKind::Listing => lsp_types::SymbolKind::METHOD,
        symbols::SymbolKind::Enumeration => lsp_types::SymbolKind::ENUM,
        symbols::SymbolKind::EnumerationItem => lsp_types::SymbolKind::ENUM_MEMBER,
        symbols::SymbolKind::Theorem => lsp_types::SymbolKind::VARIABLE,
        symbols::SymbolKind::Equation => lsp_types::SymbolKind::CONSTANT,
        symbols::SymbolKind::Entry(category) => match category {
            BibtexEntryTypeCategory::Misc => lsp_types::SymbolKind::INTERFACE,
            BibtexEntryTypeCategory::String => lsp_types::SymbolKind::STRING,
            BibtexEntryTypeCategory::Article => lsp_types::SymbolKind::EVENT,
            BibtexEntryTypeCategory::Thesis => lsp_types::SymbolKind::OBJECT,
            BibtexEntryTypeCategory::Book => lsp_types::SymbolKind::STRUCT,
            BibtexEntryTypeCategory::Part => lsp_types::SymbolKind::OPERATOR,
            BibtexEntryTypeCategory::Collection => lsp_types::SymbolKind::TYPE_PARAMETER,
        },
        symbols::SymbolKind::Field => lsp_types::SymbolKind::FIELD,
    }
}
