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
            for entry in tree
                .entries()
                .iter()
                .filter(|entry| !entry.is_comment())
                .filter(|entry| entry.key.is_some())
            {
                let key = entry.key.as_ref().unwrap();
                let symbol = DocumentSymbol {
                    name: key.text().to_owned().into(),
                    detail: None,
                    kind: SymbolKind::Interface,
                    deprecated: Some(false),
                    range: entry.range(),
                    selection_range: key.range(),
                    children: Some(Self::field_symbols(&entry)),
                };
                symbols.push(symbol);
            }
        }
        symbols
    }
}

impl BibtexEntrySymbolProvider {
    fn field_symbols(entry: &BibtexEntry) -> Vec<DocumentSymbol> {
        let mut symbols = Vec::new();
        for field in &entry.fields {
            let symbol = DocumentSymbol {
                name: field.name.text().to_owned().into(),
                detail: None,
                kind: SymbolKind::Field,
                deprecated: Some(false),
                range: field.range(),
                selection_range: field.name.range(),
                children: None,
            };
            symbols.push(symbol);
        }
        symbols
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::range::RangeExt;

    #[test]
    fn test_entry() {
        let symbols = test_feature(
            BibtexEntrySymbolProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.bib",
                    "@article{key, foo = bar, baz = qux}",
                )],
                main_file: "foo.bib",
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            symbols,
            vec![DocumentSymbol {
                name: "key".into(),
                detail: None,
                kind: SymbolKind::Interface,
                deprecated: Some(false),
                range: Range::new_simple(0, 0, 0, 35),
                selection_range: Range::new_simple(0, 9, 0, 12),
                children: Some(vec![
                    DocumentSymbol {
                        name: "foo".into(),
                        detail: None,
                        kind: SymbolKind::Field,
                        deprecated: Some(false),
                        range: Range::new_simple(0, 14, 0, 24),
                        selection_range: Range::new_simple(0, 14, 0, 17),
                        children: None,
                    },
                    DocumentSymbol {
                        name: "baz".into(),
                        detail: None,
                        kind: SymbolKind::Field,
                        deprecated: Some(false),
                        range: Range::new_simple(0, 25, 0, 34),
                        selection_range: Range::new_simple(0, 25, 0, 28),
                        children: None,
                    },
                ]),
            }]
        );
    }

    #[test]
    fn test_comment() {
        let symbols = test_feature(
            BibtexEntrySymbolProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.bib",
                    "@comment{key, foo = bar, baz = qux}",
                )],
                main_file: "foo.bib",
                ..FeatureSpec::default()
            },
        );
        assert_eq!(symbols, Vec::new());
    }

    #[test]
    fn test_latex() {
        let symbols = test_feature(
            BibtexEntrySymbolProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.tex",
                    "@article{key, foo = bar, baz = qux}",
                )],
                main_file: "foo.tex",
                ..FeatureSpec::default()
            },
        );
        assert_eq!(symbols, Vec::new());
    }
}
