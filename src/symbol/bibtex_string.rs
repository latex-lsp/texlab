use crate::syntax::*;
use crate::workspace::*;
use futures_boxed::boxed;
use lsp_types::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BibtexStringSymbolProvider;

impl FeatureProvider for BibtexStringSymbolProvider {
    type Params = DocumentSymbolParams;
    type Output = Vec<DocumentSymbol>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut symbols = Vec::new();
        if let SyntaxTree::Bibtex(tree) = &request.document().tree {
            for child in &tree.root.children {
                if let BibtexDeclaration::String(string) = &child {
                    if let Some(name) = &string.name {
                        symbols.push(DocumentSymbol {
                            name: name.text().to_owned().into(),
                            detail: None,
                            kind: SymbolKind::String,
                            deprecated: Some(false),
                            range: string.range(),
                            selection_range: name.range(),
                            children: None,
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

    #[test]
    fn test_valid() {
        let symbols = test_feature(
            BibtexStringSymbolProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@string{key = \"value\"}")],
                main_file: "foo.bib",
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            symbols,
            vec![DocumentSymbol {
                name: "key".into(),
                detail: None,
                kind: SymbolKind::String,
                deprecated: Some(false),
                range: Range::new_simple(0, 0, 0, 22),
                selection_range: Range::new_simple(0, 8, 0, 11),
                children: None,
            }]
        );
    }

    #[test]
    fn test_invalid() {
        let symbols = test_feature(
            BibtexStringSymbolProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@string{}")],
                main_file: "foo.bib",
                ..FeatureSpec::default()
            },
        );
        assert_eq!(symbols, Vec::new());
    }
}
