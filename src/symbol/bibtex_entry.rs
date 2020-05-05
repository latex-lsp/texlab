use super::types::{LatexSymbol, LatexSymbolKind};
use crate::{
    feature::{FeatureProvider, FeatureRequest},
    protocol::DocumentSymbolParams,
    syntax::{bibtex, BibtexEntryTypeCategory, SyntaxNode, LANGUAGE_DATA},
    workspace::DocumentContent,
};
use async_trait::async_trait;
use petgraph::graph::NodeIndex;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct BibtexEntrySymbolProvider;

#[async_trait]
impl FeatureProvider for BibtexEntrySymbolProvider {
    type Params = DocumentSymbolParams;
    type Output = Vec<LatexSymbol>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut symbols = Vec::new();
        if let DocumentContent::Bibtex(tree) = &req.current().content {
            for entry_node in tree.children(tree.root) {
                if let Some(entry) = tree
                    .as_entry(entry_node)
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
                        children: Self::field_symbols(tree, entry_node),
                    };
                    symbols.push(symbol);
                }
            }
        }
        symbols
    }
}

impl BibtexEntrySymbolProvider {
    fn field_symbols(tree: &bibtex::Tree, entry_node: NodeIndex) -> Vec<LatexSymbol> {
        let mut symbols = Vec::new();
        for field in tree
            .children(entry_node)
            .filter_map(|node| tree.as_field(node))
        {
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
    use crate::{
        feature::FeatureTester,
        protocol::{Range, RangeExt},
    };

    #[tokio::test]
    async fn empty_latex_document() {
        let actual_symbols = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .test_symbol(BibtexEntrySymbolProvider)
            .await;

        assert!(actual_symbols.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_symbols = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .test_symbol(BibtexEntrySymbolProvider)
            .await;

        assert!(actual_symbols.is_empty());
    }

    #[tokio::test]
    async fn entry() {
        let actual_symbols = FeatureTester::new()
            .file("main.bib", r#"@article{key, foo = bar, baz = qux}"#)
            .main("main.bib")
            .test_symbol(BibtexEntrySymbolProvider)
            .await;

        let expected_symbols = vec![LatexSymbol {
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
        }];

        assert_eq!(actual_symbols, expected_symbols);
    }

    #[tokio::test]
    async fn comment() {
        let actual_symbols = FeatureTester::new()
            .file("main.bib", r#"@comment{key, foo = bar, baz = qux}"#)
            .main("main.bib")
            .test_symbol(BibtexEntrySymbolProvider)
            .await;

        assert!(actual_symbols.is_empty());
    }
}
