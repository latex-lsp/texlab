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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LatexSymbolKind {
    Section,
    Figure,
    Algorithm,
    Table,
    Listing,
    Theorem,
    Equation,
    Entry,
    Field,
    String,
}

impl Into<SymbolKind> for LatexSymbolKind {
    fn into(self) -> SymbolKind {
        match self {
            Self::Section => SymbolKind::Module,
            Self::Figure | Self::Algorithm | Self::Table | Self::Listing => SymbolKind::Method,
            Self::Theorem => SymbolKind::Class,
            Self::Equation => SymbolKind::Constant,
            Self::Entry => SymbolKind::Interface,
            Self::Field => SymbolKind::Field,
            Self::String => SymbolKind::String,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexSymbol {
    pub name: String,
    pub label: Option<String>,
    pub kind: LatexSymbolKind,
    pub deprecated: bool,
    pub full_range: Range,
    pub selection_range: Range,
    pub children: Vec<LatexSymbol>,
}

impl LatexSymbol {
    pub fn search_text(&self) -> String {
        let kind = match self.kind {
            LatexSymbolKind::Section => "section",
            LatexSymbolKind::Figure => "figure",
            LatexSymbolKind::Algorithm => "algorithm",
            LatexSymbolKind::Table => "table",
            LatexSymbolKind::Listing => "listing",
            LatexSymbolKind::Theorem => "math",
            LatexSymbolKind::Equation => "math equation",
            LatexSymbolKind::Entry => "entry",
            LatexSymbolKind::Field => "field",
            LatexSymbolKind::String => "string",
        };
        format!("{} {}", kind, self.name).to_lowercase()
    }

    pub fn flatten(mut self, buffer: &mut Vec<Self>) {
        if self.kind == LatexSymbolKind::Field {
            return;
        }
        for symbol in self.children.drain(..) {
            symbol.flatten(buffer);
        }
        buffer.push(self);
    }

    pub fn into_symbol_info(self, uri: Uri) -> SymbolInformation {
        SymbolInformation {
            name: self.name,
            deprecated: Some(self.deprecated),
            kind: self.kind.into(),
            container_name: None,
            location: Location::new(uri.clone().into(), self.full_range),
        }
    }
}

impl Into<DocumentSymbol> for LatexSymbol {
    fn into(self) -> DocumentSymbol {
        let children = self.children.into_iter().map(Into::into).collect();
        DocumentSymbol {
            name: self.name,
            deprecated: Some(self.deprecated),
            detail: self.label,
            kind: self.kind.into(),
            selection_range: self.selection_range,
            range: self.full_range,
            children: Some(children),
        }
    }
}

pub struct SymbolProvider {
    provider: ConcatProvider<DocumentSymbolParams, LatexSymbol>,
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
    type Output = Vec<LatexSymbol>;

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
        symbols: Vec<LatexSymbol>,
    ) -> Self {
        if client_capabilities.has_hierarchical_document_symbol_support() {
            Self::Hierarchical(symbols.into_iter().map(Into::into).collect())
        } else {
            let mut buffer = Vec::new();
            for symbol in symbols {
                symbol.flatten(&mut buffer);
            }
            Self::Flat(
                buffer
                    .into_iter()
                    .map(|symbol| symbol.into_symbol_info(uri.clone()))
                    .collect(),
            )
        }
    }
}

struct WorkspaceSymbol {
    info: SymbolInformation,
    search_text: String,
}

pub async fn workspace_symbols(
    client_capabilities: Arc<ClientCapabilities>,
    workspace: Arc<Workspace>,
    params: &WorkspaceSymbolParams,
) -> Vec<SymbolInformation> {
    let provider = SymbolProvider::new();
    let mut symbols = Vec::new();

    for document in &workspace.documents {
        let uri: Uri = document.uri.clone();
        let request = FeatureRequest {
            client_capabilities: Arc::clone(&client_capabilities),
            view: DocumentView::new(Arc::clone(&workspace), Arc::clone(&document)),
            params: DocumentSymbolParams {
                text_document: TextDocumentIdentifier::new(uri.clone().into()),
            },
        };

        let mut buffer = Vec::new();
        for symbol in provider.execute(&request).await {
            symbol.flatten(&mut buffer);
        }

        for symbol in buffer {
            symbols.push(WorkspaceSymbol {
                search_text: symbol.search_text(),
                info: symbol.into_symbol_info(uri.clone()),
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
    filtered
}
