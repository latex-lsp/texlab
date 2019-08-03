use crate::syntax::*;
use crate::workspace::*;
use futures_boxed::boxed;
use lsp_types::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BibtexEntrySymbolProvider;

impl FeatureProvider for BibtexEntrySymbolProvider {
    type Params = DocumentSymbolParams;
    type Output = Vec<DocumentSymbol>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut symbols = Vec::new();
        if let SyntaxTree::Bibtex(tree) = &request.document().tree {
            for entry in tree.entries() {
                if !entry.is_comment() {
                    if let Some(key) = &entry.key {
                        let symbol = DocumentSymbol {
                            name: key.text().to_owned().into(),
                            detail: None,
                            kind: SymbolKind::Interface,
                            deprecated: Some(false),
                            range: entry.range(),
                            selection_range: key.range(),
                            children: None,
                        };
                        symbols.push(symbol);
                    }
                }
            }
        }
        symbols
    }
}
