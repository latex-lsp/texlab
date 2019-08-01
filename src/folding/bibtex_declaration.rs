use crate::syntax::*;
use crate::workspace::*;
use futures_boxed::boxed;
use lsp_types::{FoldingRange, FoldingRangeKind, FoldingRangeParams};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BibtexDeclarationFoldingProvider;

impl FeatureProvider for BibtexDeclarationFoldingProvider {
    type Params = FoldingRangeParams;
    type Output = Vec<FoldingRange>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<FoldingRangeParams>,
    ) -> Vec<FoldingRange> {
        if let SyntaxTree::Bibtex(tree) = &request.document().tree {
            tree.root.children.iter().flat_map(Self::fold).collect()
        } else {
            Vec::new()
        }
    }
}

impl BibtexDeclarationFoldingProvider {
    fn fold(declaration: &BibtexDeclaration) -> Option<FoldingRange> {
        let ty = match declaration {
            BibtexDeclaration::Comment(_) => None,
            BibtexDeclaration::Preamble(preamble) => Some(&preamble.ty),
            BibtexDeclaration::String(string) => Some(&string.ty),
            BibtexDeclaration::Entry(entry) => Some(&entry.ty),
        }?;

        let right = match declaration {
            BibtexDeclaration::Comment(_) => None,
            BibtexDeclaration::Preamble(preamble) => preamble.right.as_ref(),
            BibtexDeclaration::String(string) => string.right.as_ref(),
            BibtexDeclaration::Entry(entry) => entry.right.as_ref(),
        }?;

        Some(FoldingRange {
            start_line: ty.range().start.line,
            start_character: Some(ty.range().start.character),
            end_line: right.range().start.line,
            end_character: Some(right.range().start.character),
            kind: Some(FoldingRangeKind::Region),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preamble() {
        let foldings = test_feature(
            BibtexDeclarationFoldingProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "\n@preamble{\"foo\"}")],
                main_file: "foo.bib",
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            foldings,
            vec![FoldingRange {
                start_line: 1,
                start_character: Some(0),
                end_line: 1,
                end_character: Some(15),
                kind: Some(FoldingRangeKind::Region),
            }]
        );
    }

    #[test]
    fn test_string() {
        let foldings = test_feature(
            BibtexDeclarationFoldingProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@string{foo = \"bar\"}")],
                main_file: "foo.bib",
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            foldings,
            vec![FoldingRange {
                start_line: 0,
                start_character: Some(0),
                end_line: 0,
                end_character: Some(19),
                kind: Some(FoldingRangeKind::Region),
            }]
        );
    }

    #[test]
    fn test_entry() {
        let foldings = test_feature(
            BibtexDeclarationFoldingProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, bar = baz\n}")],
                main_file: "foo.bib",
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            foldings,
            vec![FoldingRange {
                start_line: 0,
                start_character: Some(0),
                end_line: 1,
                end_character: Some(0),
                kind: Some(FoldingRangeKind::Region),
            }]
        );
    }

    #[test]
    fn test_comment() {
        let foldings = test_feature(
            BibtexDeclarationFoldingProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "foo")],
                main_file: "foo.bib",
                ..FeatureSpec::default()
            },
        );
        assert!(foldings.is_empty());
    }

    #[test]
    fn test_entry_invalid() {
        let foldings = test_feature(
            BibtexDeclarationFoldingProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo,")],
                main_file: "foo.bib",
                ..FeatureSpec::default()
            },
        );
        assert!(foldings.is_empty());
    }

    #[test]
    fn test_latex() {
        let foldings = test_feature(
            BibtexDeclarationFoldingProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "")],
                main_file: "foo.tex",
                ..FeatureSpec::default()
            },
        );
        assert!(foldings.is_empty());
    }
}
