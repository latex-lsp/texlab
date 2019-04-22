use crate::feature::FeatureRequest;
use crate::syntax::bibtex::ast::BibtexDeclaration;
use crate::syntax::text::SyntaxNode;
use crate::workspace::SyntaxTree;
use lsp_types::{FoldingRange, FoldingRangeKind, FoldingRangeParams};

pub struct BibtexDeclarationFoldingProvider;

impl BibtexDeclarationFoldingProvider {
    pub async fn execute(request: &FeatureRequest<FoldingRangeParams>) -> Vec<FoldingRange> {
        if let SyntaxTree::Bibtex(tree) = &request.document.tree {
            tree.root.children.iter().flat_map(Self::fold).collect()
        } else {
            Vec::new()
        }
    }

    fn fold(declaration: &BibtexDeclaration) -> Option<FoldingRange> {
        let kind = match declaration {
            BibtexDeclaration::Comment(_) => None,
            BibtexDeclaration::Preamble(preamble) => Some(&preamble.kind),
            BibtexDeclaration::String(string) => Some(&string.kind),
            BibtexDeclaration::Entry(entry) => Some(&entry.kind),
        }?;

        let right = match declaration {
            BibtexDeclaration::Comment(_) => None,
            BibtexDeclaration::Preamble(preamble) => preamble.right.as_ref(),
            BibtexDeclaration::String(string) => string.right.as_ref(),
            BibtexDeclaration::Entry(entry) => entry.right.as_ref(),
        }?;

        Some(FoldingRange {
            start_line: kind.range().start.line,
            start_character: Some(kind.range().start.character),
            end_line: right.range().start.line,
            end_character: Some(right.range().start.character),
            kind: Some(FoldingRangeKind::Region),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;
    use crate::workspace::WorkspaceBuilder;
    use futures::executor;

    #[test]
    fn test_preamble() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.bib", "\n@preamble{\"foo\"}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 0, "").into();

        let results = executor::block_on(BibtexDeclarationFoldingProvider::execute(&request));

        let folding = FoldingRange {
            start_line: 1,
            start_character: Some(0),
            end_line: 1,
            end_character: Some(15),
            kind: Some(FoldingRangeKind::Region),
        };
        assert_eq!(vec![folding], results);
    }

    #[test]
    fn test_string() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.bib", "@string{foo = \"bar\"}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 0, "").into();

        let results = executor::block_on(BibtexDeclarationFoldingProvider::execute(&request));

        let folding = FoldingRange {
            start_line: 0,
            start_character: Some(0),
            end_line: 0,
            end_character: Some(19),
            kind: Some(FoldingRangeKind::Region),
        };
        assert_eq!(vec![folding], results);
    }

    #[test]
    fn test_entry() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.bib", "@article{foo, bar = baz\n}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 0, "").into();

        let results = executor::block_on(BibtexDeclarationFoldingProvider::execute(&request));

        let folding = FoldingRange {
            start_line: 0,
            start_character: Some(0),
            end_line: 1,
            end_character: Some(0),
            kind: Some(FoldingRangeKind::Region),
        };
        assert_eq!(vec![folding], results);
    }

    #[test]
    fn test_comment() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.bib", "foo");
        let request = FeatureTester::new(builder.workspace, uri, 0, 0, "").into();

        let results = executor::block_on(BibtexDeclarationFoldingProvider::execute(&request));

        assert_eq!(results, Vec::new());
    }

    #[test]
    fn test_entry_invalid() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.bib", "@article{foo,");
        let request = FeatureTester::new(builder.workspace, uri, 0, 0, "").into();

        let results = executor::block_on(BibtexDeclarationFoldingProvider::execute(&request));

        assert_eq!(results, Vec::new());
    }

    #[test]
    fn test_latex() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "@article{foo,");
        let request = FeatureTester::new(builder.workspace, uri, 0, 0, "").into();

        let results = executor::block_on(BibtexDeclarationFoldingProvider::execute(&request));

        assert_eq!(results, Vec::new());
    }
}
