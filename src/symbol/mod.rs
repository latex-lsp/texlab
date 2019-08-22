mod bibtex_entry;
mod bibtex_string;
mod latex_section;

use self::bibtex_entry::BibtexEntrySymbolProvider;
use self::bibtex_string::BibtexStringSymbolProvider;
use self::latex_section::LatexSectionSymbolProvider;
use crate::workspace::*;
use futures_boxed::boxed;
use lsp_types::*;
use serde::{Deserialize, Serialize};

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
        let supports_hierarchical = client_capabilities
            .text_document
            .as_ref()
            .and_then(|cap| cap.document_symbol.as_ref())
            .and_then(|cap| cap.hierarchical_document_symbol_support)
            == Some(true);

        if supports_hierarchical {
            Self::Hierarchical(symbols)
        } else {
            fn flatten(results: &mut Vec<SymbolInformation>, uri: &Uri, symbol: DocumentSymbol) {
                let info = SymbolInformation {
                    name: symbol.name,
                    deprecated: Some(false),
                    kind: symbol.kind,
                    container_name: None,
                    location: Location::new(uri.clone(), symbol.range),
                };
                results.push(info);
                if let Some(children) = symbol.children {
                    for child in children {
                        flatten(results, uri, child);
                    }
                }
            }
            let mut results = Vec::new();
            for symbol in symbols {
                flatten(&mut results, uri, symbol);
            }
            Self::Flat(results)
        }
    }
}
