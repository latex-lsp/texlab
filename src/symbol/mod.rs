mod bibtex_entry;
mod bibtex_string;
mod latex_section;

use self::bibtex_entry::BibtexEntrySymbolProvider;
use self::bibtex_string::BibtexStringSymbolProvider;
use self::latex_section::LatexSectionSymbolProvider;
use crate::capabilities::ClientCapabilitiesExt;
use crate::workspace::*;
use futures_boxed::boxed;
use lsp_types::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct SymbolProvider {
    provider: ConcatProvider<DocumentSymbolParams, DocumentSymbol>,
}

impl SymbolProvider {
    pub fn new() -> Self {
        Self {
            provider: ConcatProvider::new(vec![
                Box::new(BibtexEntrySymbolProvider),
                Box::new(BibtexStringSymbolProvider),
                Box::new(LatexSectionSymbolProvider),
            ]),
        }
    }
}

impl Default for SymbolProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureProvider for SymbolProvider {
    type Params = DocumentSymbolParams;
    type Output = Vec<DocumentSymbol>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        self.provider.execute(request).await
    }
}

#[serde(untagged)]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolResponse {
    Flat(Vec<SymbolInformation>),
    Hierarchical(Vec<DocumentSymbol>),
}

impl SymbolResponse {
    pub fn new(
        client_capabilities: &ClientCapabilities,
        uri: &Uri,
        symbols: Vec<DocumentSymbol>,
    ) -> Self {
        if client_capabilities.has_hierarchical_document_symbol_support() {
            Self::Hierarchical(symbols)
        } else {
            let mut results = Vec::new();
            for symbol in symbols {
                Self::flatten(&mut results, uri, symbol);
            }
            Self::Flat(results)
        }
    }

    fn flatten(results: &mut Vec<SymbolInformation>, uri: &Uri, symbol: DocumentSymbol) {
        if symbol.kind == SymbolKind::Field {
            return;
        }

        let info = SymbolInformation {
            name: symbol.name,
            deprecated: Some(false),
            kind: symbol.kind,
            container_name: None,
            location: Location::new(uri.clone().into(), symbol.range),
        };
        results.push(info);
        if let Some(children) = symbol.children {
            for child in children {
                Self::flatten(results, uri, child);
            }
        }
    }
}

pub async fn workspace_symbols(
    client_capabilities: Arc<ClientCapabilities>,
    workspace: Arc<Workspace>,
    params: &WorkspaceSymbolParams,
) -> Vec<SymbolInformation> {
    let provider = SymbolProvider::new();
    let mut all_symbols = Vec::new();

    for document in &workspace.documents {
        let uri: Uri = document.uri.clone();
        let request = FeatureRequest {
            client_capabilities: Arc::clone(&client_capabilities),
            view: DocumentView::new(Arc::clone(&workspace), Arc::clone(&document)),
            params: DocumentSymbolParams {
                text_document: TextDocumentIdentifier::new(uri.clone().into()),
            },
        };
        for symbol in provider.execute(&request).await {
            SymbolResponse::flatten(&mut all_symbols, &uri, symbol);
        }
    }

    let query_words: Vec<String> = params
        .query
        .split_whitespace()
        .map(str::to_lowercase)
        .collect();
    let mut filtered_symbols = Vec::new();
    for symbol in all_symbols {
        let name = symbol.name.to_lowercase();
        let mut included = true;
        for word in &query_words {
            if !name.contains(word) {
                included = false;
                break;
            }
        }

        if included {
            filtered_symbols.push(symbol);
        }
    }
    filtered_symbols
}
