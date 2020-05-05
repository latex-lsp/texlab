use super::types::{LatexSymbol, LatexSymbolKind};
use crate::{
    feature::{FeatureProvider, FeatureRequest},
    protocol::DocumentSymbolParams,
    syntax::SyntaxNode,
    workspace::DocumentContent,
};
use async_trait::async_trait;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct BibtexStringSymbolProvider;

#[async_trait]
impl FeatureProvider for BibtexStringSymbolProvider {
    type Params = DocumentSymbolParams;
    type Output = Vec<LatexSymbol>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut symbols = Vec::new();
        if let DocumentContent::Bibtex(tree) = &req.current().content {
            for string_node in tree.children(tree.root) {
                if let Some(string) = &tree.as_string(string_node) {
                    if let Some(name) = &string.name {
                        symbols.push(LatexSymbol {
                            name: name.text().into(),
                            label: None,
                            kind: LatexSymbolKind::String,
                            deprecated: false,
                            full_range: string.range(),
                            selection_range: name.range(),
                            children: Vec::new(),
                        });
                    }
                }
            }
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
            .test_symbol(BibtexStringSymbolProvider)
            .await;

        assert!(actual_symbols.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_symbols = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .test_symbol(BibtexStringSymbolProvider)
            .await;

        assert!(actual_symbols.is_empty());
    }

    #[tokio::test]
    async fn valid() {
        let actual_symbols = FeatureTester::new()
            .file("main.bib", r#"@string{key = "value"}"#)
            .main("main.bib")
            .test_symbol(BibtexStringSymbolProvider)
            .await;

        let expected_symbols = vec![LatexSymbol {
            name: "key".into(),
            label: None,
            kind: LatexSymbolKind::String,
            deprecated: false,
            full_range: Range::new_simple(0, 0, 0, 22),
            selection_range: Range::new_simple(0, 8, 0, 11),
            children: Vec::new(),
        }];

        assert_eq!(actual_symbols, expected_symbols);
    }

    #[tokio::test]
    async fn invalid() {
        let actual_symbols = FeatureTester::new()
            .file("main.bib", r#"@string{}"#)
            .main("main.bib")
            .test_symbol(BibtexStringSymbolProvider)
            .await;

        assert!(actual_symbols.is_empty());
    }
}
