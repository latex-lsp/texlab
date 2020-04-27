mod bibtex_entry;
mod bibtex_string;
mod latex_section;
mod project_order;
mod types;

pub use self::latex_section::{build_section_tree, LatexSectionNode, LatexSectionTree};

use self::{
    bibtex_entry::BibtexEntrySymbolProvider, bibtex_string::BibtexStringSymbolProvider,
    latex_section::LatexSectionSymbolProvider, project_order::ProjectOrdering, types::LatexSymbol,
};
use futures_boxed::boxed;
use std::{
    cmp::Reverse,
    path::{Path, PathBuf},
    sync::Arc,
};
use texlab_feature::{ConcatProvider, DocumentView, FeatureProvider, FeatureRequest, Snapshot};
use texlab_protocol::{
    ClientCapabilities, ClientCapabilitiesExt, DocumentSymbolParams, DocumentSymbolResponse,
    Options, PartialResultParams, SymbolInformation, TextDocumentIdentifier, Uri,
    WorkDoneProgressParams, WorkspaceSymbolParams,
};
use texlab_tex::DynamicDistribution;

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
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        self.provider.execute(req).await
    }
}

pub fn document_symbols(
    client_capabilities: &ClientCapabilities,
    snapshot: &Snapshot,
    uri: &Uri,
    options: &Options,
    current_dir: &Path,
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
        sort_symbols(snapshot, options, &current_dir, &mut buffer);
        DocumentSymbolResponse::Flat(buffer)
    }
}

struct WorkspaceSymbol {
    info: SymbolInformation,
    search_text: String,
}

pub async fn workspace_symbols<'a>(
    distro: DynamicDistribution,
    client_capabilities: Arc<ClientCapabilities>,
    snapshot: Arc<Snapshot>,
    options: &'a Options,
    current_dir: Arc<PathBuf>,
    params: &'a WorkspaceSymbolParams,
) -> Vec<SymbolInformation> {
    let provider = SymbolProvider::new();
    let mut symbols = Vec::new();

    for doc in &snapshot.0 {
        let uri: Uri = doc.uri.clone();
        let req = FeatureRequest {
            params: DocumentSymbolParams {
                text_document: TextDocumentIdentifier::new(uri.clone().into()),
                work_done_progress_params: WorkDoneProgressParams::default(),
                partial_result_params: PartialResultParams::default(),
            },
            view: DocumentView::analyze(
                Arc::clone(&snapshot),
                Arc::clone(&doc),
                &options,
                &current_dir,
            ),
            distro: distro.clone(),
            client_capabilities: Arc::clone(&client_capabilities),
            options: options.clone(),
            current_dir: Arc::clone(&current_dir),
        };

        let mut buffer = Vec::new();
        for symbol in provider.execute(&req).await {
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
    sort_symbols(&snapshot, options, &current_dir, &mut filtered);
    filtered
}

fn sort_symbols(
    snapshot: &Snapshot,
    options: &Options,
    current_dir: &Path,
    symbols: &mut Vec<SymbolInformation>,
) {
    let ordering = ProjectOrdering::analyze(snapshot, options, current_dir);
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
