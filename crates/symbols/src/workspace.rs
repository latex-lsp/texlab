mod sort;

use std::cmp::Reverse;

use base_db::Workspace;

use crate::{document_symbols, types::SymbolLocation, SymbolKind};

use self::sort::ProjectOrdering;

pub fn workspace_symbols<'a>(workspace: &'a Workspace, query: &str) -> Vec<SymbolLocation<'a>> {
    let query = query.split_whitespace().collect::<Vec<_>>();
    let mut results = Vec::new();

    for document in workspace.iter() {
        let mut buf = Vec::new();
        let symbols = document_symbols(workspace, document);
        for symbol in symbols {
            symbol.flatten(&mut buf);
        }

        for symbol in buf
            .into_iter()
            .filter(|symbol| symbol.kind != SymbolKind::Field)
        {
            let keywords = symbol.keywords();
            if query.is_empty()
                || itertools::iproduct!(keywords.iter(), query.iter())
                    .any(|(keyword, query)| keyword.eq_ignore_ascii_case(query))
            {
                results.push(SymbolLocation { document, symbol });
            }
        }
    }

    let ordering = ProjectOrdering::from(workspace);
    results.sort_by_key(|item| {
        let index = ordering.get(&item.document.uri);
        let range = item.symbol.full_range;
        (index, range.start(), Reverse(range.end()))
    });

    results
}

#[cfg(test)]
mod tests;
