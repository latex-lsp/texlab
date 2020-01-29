mod bibtex_entry;
mod bibtex_string;
mod latex_section;
mod project_order;

use self::bibtex_entry::BibtexEntrySymbolProvider;
use self::bibtex_string::BibtexStringSymbolProvider;
use self::latex_section::LatexSectionSymbolProvider;
use self::project_order::ProjectOrdering;
use futures_boxed::boxed;
use std::cmp::Reverse;
use std::sync::Arc;
use texlab_distro::Distribution;
use texlab_protocol::ClientCapabilitiesExt;
use texlab_protocol::*;
use texlab_syntax::*;
use texlab_workspace::*;

pub use self::latex_section::{build_section_tree, LatexSectionNode, LatexSectionTree};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LatexSymbolKind {
    Section,
    Figure,
    Algorithm,
    Table,
    Listing,
    Enumeration,
    EnumerationItem,
    Theorem,
    Equation,
    Entry(BibtexEntryTypeCategory),
    Field,
    String,
}

impl Into<SymbolKind> for LatexSymbolKind {
    fn into(self) -> SymbolKind {
        match self {
            Self::Section => Structure::Section.symbol_kind(),
            Self::Figure | Self::Algorithm | Self::Table | Self::Listing => {
                Structure::Float.symbol_kind()
            }
            Self::Enumeration => Structure::Environment.symbol_kind(),
            Self::EnumerationItem => Structure::Item.symbol_kind(),
            Self::Theorem => Structure::Theorem.symbol_kind(),
            Self::Equation => Structure::Equation.symbol_kind(),
            Self::Entry(category) => Structure::Entry(category).symbol_kind(),
            Self::Field => Structure::Field.symbol_kind(),
            Self::String => Structure::Entry(BibtexEntryTypeCategory::String).symbol_kind(),
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
            LatexSymbolKind::Section => "latex section",
            LatexSymbolKind::Figure => "latex float figure",
            LatexSymbolKind::Algorithm => "latex float algorithm",
            LatexSymbolKind::Table => "latex float table",
            LatexSymbolKind::Listing => "latex float listing",
            LatexSymbolKind::Enumeration => "latex enumeration",
            LatexSymbolKind::EnumerationItem => "latex enumeration item",
            LatexSymbolKind::Theorem => "latex math",
            LatexSymbolKind::Equation => "latex math equation",
            LatexSymbolKind::Entry(_) => "bibtex entry",
            LatexSymbolKind::Field => "bibtex field",
            LatexSymbolKind::String => "bibtex string",
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

pub fn document_symbols(
    client_capabilities: &ClientCapabilities,
    workspace: &Workspace,
    uri: &Uri,
    options: &Options,
    symbols: Vec<LatexSymbol>,
) -> DocumentSymbolResponse {
    if client_capabilities.has_hierarchical_document_symbol_support() {
        DocumentSymbolResponse::Nested(symbols.into_iter().map(Into::into).collect())
    } else {
        let mut buffer = Vec::new();
        for symbol in symbols {
            symbol.flatten(&mut buffer);
        }
        let mut buffer = buffer
            .into_iter()
            .map(|symbol| symbol.into_symbol_info(uri.clone()))
            .collect();
        sort_symbols(workspace, options, &mut buffer);
        DocumentSymbolResponse::Flat(buffer)
    }
}

struct WorkspaceSymbol {
    info: SymbolInformation,
    search_text: String,
}

pub async fn workspace_symbols<'a>(
    distribution: Arc<Box<dyn Distribution>>,
    client_capabilities: Arc<ClientCapabilities>,
    workspace: Arc<Workspace>,
    options: &'a Options,
    params: &'a WorkspaceSymbolParams,
) -> Vec<SymbolInformation> {
    let provider = SymbolProvider::new();
    let mut symbols = Vec::new();

    for document in &workspace.documents {
        let uri: Uri = document.uri.clone();
        let request = FeatureRequest {
            client_capabilities: Arc::clone(&client_capabilities),
            view: DocumentView::new(Arc::clone(&workspace), Arc::clone(&document), options),
            params: DocumentSymbolParams {
                text_document: TextDocumentIdentifier::new(uri.clone().into()),
            },
            distribution: Arc::clone(&distribution),
            options: Options::default(),
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
    sort_symbols(&workspace, options, &mut filtered);
    filtered
}

fn sort_symbols(workspace: &Workspace, options: &Options, symbols: &mut Vec<SymbolInformation>) {
    let ordering = ProjectOrdering::new(workspace, options);
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
