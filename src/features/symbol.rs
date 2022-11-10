mod bibtex;
mod latex;
mod project_order;
mod types;

use std::cmp::Reverse;

use lsp_types::{DocumentSymbolResponse, SymbolInformation, Url, WorkspaceSymbolParams};

use crate::{db::workspace::Workspace, util::capabilities::ClientCapabilitiesExt, Db};

use self::project_order::ProjectOrdering;

pub fn find_document_symbols(db: &dyn Db, uri: &Url) -> Option<DocumentSymbolResponse> {
    let workspace = Workspace::get(db);
    let document = workspace.lookup_uri(db, uri)?;

    let mut buf = Vec::new();
    latex::find_symbols(db, document, &mut buf);
    bibtex::find_symbols(db, document, &mut buf);
    if workspace
        .client_capabilities(db)
        .has_hierarchical_document_symbol_support()
    {
        let symbols = buf
            .into_iter()
            .map(|symbol| symbol.into_document_symbol(db))
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

        sort_symbols(db, &mut new_buf);
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
    db: &dyn Db,
    params: &WorkspaceSymbolParams,
) -> Vec<SymbolInformation> {
    let mut symbols = Vec::new();

    let workspace = Workspace::get(db);
    for document in workspace.documents(db).iter().copied() {
        let mut buf = Vec::new();
        latex::find_symbols(db, document, &mut buf);
        bibtex::find_symbols(db, document, &mut buf);
        let mut new_buf = Vec::new();

        for symbol in buf {
            symbol.flatten(&mut new_buf);
        }

        for symbol in new_buf {
            symbols.push(WorkspaceSymbol {
                search_text: symbol.search_text(),
                info: symbol.into_symbol_info(document.location(db).uri(db).clone()),
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

    sort_symbols(db, &mut filtered);
    filtered
}

fn sort_symbols(db: &dyn Db, symbols: &mut [SymbolInformation]) {
    let ordering = ProjectOrdering::new(db);
    symbols.sort_by(|left, right| {
        let left_key = (
            ordering.get(db, &left.location.uri),
            left.location.range.start,
            Reverse(left.location.range.end),
        );
        let right_key = (
            ordering.get(db, &right.location.uri),
            right.location.range.start,
            Reverse(right.location.range.end),
        );
        left_key.cmp(&right_key)
    });
}
