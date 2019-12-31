use super::{LatexSymbol, LatexSymbolKind};
use futures_boxed::boxed;
use texlab_protocol::*;
use texlab_syntax::*;
use texlab_workspace::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BibtexStringSymbolProvider;

impl FeatureProvider for BibtexStringSymbolProvider {
    type Params = DocumentSymbolParams;
    type Output = Vec<LatexSymbol>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut symbols = Vec::new();
        if let SyntaxTree::Bibtex(tree) = &request.document().tree {
            for child in &tree.root.children {
                if let BibtexDeclaration::String(string) = &child {
                    if let Some(name) = &string.name {
                        symbols.push(LatexSymbol {
                            name: name.text().to_owned(),
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
    use texlab_protocol::RangeExt;

    #[test]
    fn valid() {
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
            vec![LatexSymbol {
                name: "key".into(),
                label: None,
                kind: LatexSymbolKind::String,
                deprecated: false,
                full_range: Range::new_simple(0, 0, 0, 22),
                selection_range: Range::new_simple(0, 8, 0, 11),
                children: Vec::new(),
            }]
        );
    }

    #[test]
    fn invalid() {
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
