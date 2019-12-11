use super::{LatexSymbol, LatexSymbolKind};
use crate::syntax::*;
use crate::workspace::*;
use futures_boxed::boxed;
use texlab_protocol::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BibtexEntrySymbolProvider;

impl FeatureProvider for BibtexEntrySymbolProvider {
    type Params = DocumentSymbolParams;
    type Output = Vec<LatexSymbol>;

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
                let category = LANGUAGE_DATA
                    .find_entry_type(&entry.ty.text()[1..])
                    .map(|ty| ty.category)
                    .unwrap_or(BibtexEntryTypeCategory::Misc);

                let key = entry.key.as_ref().unwrap();
                let symbol = LatexSymbol {
                    name: key.text().to_owned(),
                    label: None,
                    kind: LatexSymbolKind::Entry(category),
                    deprecated: false,
                    full_range: entry.range(),
                    selection_range: key.range(),
                    children: Self::field_symbols(&entry),
                };
                symbols.push(symbol);
            }
        }
        symbols
    }
}

impl BibtexEntrySymbolProvider {
    fn field_symbols(entry: &BibtexEntry) -> Vec<LatexSymbol> {
        let mut symbols = Vec::new();
        for field in &entry.fields {
            let symbol = LatexSymbol {
                name: field.name.text().to_owned(),
                label: None,
                kind: LatexSymbolKind::Field,
                deprecated: false,
                full_range: field.range(),
                selection_range: field.name.range(),
                children: Vec::new(),
            };
            symbols.push(symbol);
        }
        symbols
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use texlab_protocol::RangeExt;

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
            vec![LatexSymbol {
                name: "key".into(),
                label: None,
                kind: LatexSymbolKind::Entry(BibtexEntryTypeCategory::Article),
                deprecated: false,
                full_range: Range::new_simple(0, 0, 0, 35),
                selection_range: Range::new_simple(0, 9, 0, 12),
                children: vec![
                    LatexSymbol {
                        name: "foo".into(),
                        label: None,
                        kind: LatexSymbolKind::Field,
                        deprecated: false,
                        full_range: Range::new_simple(0, 14, 0, 24),
                        selection_range: Range::new_simple(0, 14, 0, 17),
                        children: Vec::new(),
                    },
                    LatexSymbol {
                        name: "baz".into(),
                        label: None,
                        kind: LatexSymbolKind::Field,
                        deprecated: false,
                        full_range: Range::new_simple(0, 25, 0, 34),
                        selection_range: Range::new_simple(0, 25, 0, 28),
                        children: Vec::new(),
                    },
                ],
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
