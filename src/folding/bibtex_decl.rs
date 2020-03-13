use crate::{
    feature::{FeatureProvider, FeatureRequest},
    protocol::{FoldingRange, FoldingRangeKind, FoldingRangeParams},
    syntax::{bibtex, SyntaxNode},
    workspace::DocumentContent,
};
use futures_boxed::boxed;
use petgraph::graph::NodeIndex;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct BibtexDeclarationFoldingProvider;

impl FeatureProvider for BibtexDeclarationFoldingProvider {
    type Params = FoldingRangeParams;
    type Output = Vec<FoldingRange>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        if let DocumentContent::Bibtex(tree) = &req.current().content {
            tree.children(tree.root)
                .filter_map(|decl| Self::fold(tree, decl))
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl BibtexDeclarationFoldingProvider {
    fn fold(tree: &bibtex::Tree, decl: NodeIndex) -> Option<FoldingRange> {
        let (ty, right) = match &tree.graph[decl] {
            bibtex::Node::Preamble(preamble) => (Some(&preamble.ty), preamble.right.as_ref()),
            bibtex::Node::String(string) => (Some(&string.ty), string.right.as_ref()),
            bibtex::Node::Entry(entry) => (Some(&entry.ty), entry.right.as_ref()),
            bibtex::Node::Root(_)
            | bibtex::Node::Comment(_)
            | bibtex::Node::Field(_)
            | bibtex::Node::Word(_)
            | bibtex::Node::Command(_)
            | bibtex::Node::QuotedContent(_)
            | bibtex::Node::BracedContent(_)
            | bibtex::Node::Concat(_) => (None, None),
        };

        Some(FoldingRange {
            start_line: ty?.start().line,
            start_character: Some(ty?.start().character),
            end_line: right?.end().line,
            end_character: Some(right?.end().character),
            kind: Some(FoldingRangeKind::Region),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;
    use indoc::indoc;

    #[tokio::test]
    async fn preamble() {
        let actual_foldings = FeatureTester::new()
            .file("main.bib", r#"@preamble{"foo"}"#)
            .main("main.bib")
            .test_folding(BibtexDeclarationFoldingProvider)
            .await;

        let expected_foldings = vec![FoldingRange {
            start_line: 0,
            start_character: Some(0),
            end_line: 0,
            end_character: Some(16),
            kind: Some(FoldingRangeKind::Region),
        }];

        assert_eq!(actual_foldings, expected_foldings);
    }

    #[tokio::test]
    async fn string() {
        let actual_foldings = FeatureTester::new()
            .file("main.bib", r#"@string{foo = "bar"}"#)
            .main("main.bib")
            .test_folding(BibtexDeclarationFoldingProvider)
            .await;

        let expected_foldings = vec![FoldingRange {
            start_line: 0,
            start_character: Some(0),
            end_line: 0,
            end_character: Some(20),
            kind: Some(FoldingRangeKind::Region),
        }];

        assert_eq!(actual_foldings, expected_foldings);
    }

    #[tokio::test]
    async fn entry() {
        let actual_foldings = FeatureTester::new()
            .file(
                "main.bib",
                indoc!(
                    r#"
                        @article{foo, 
                            bar = baz
                        }
                    "#
                ),
            )
            .main("main.bib")
            .test_folding(BibtexDeclarationFoldingProvider)
            .await;

        let expected_foldings = vec![FoldingRange {
            start_line: 0,
            start_character: Some(0),
            end_line: 2,
            end_character: Some(1),
            kind: Some(FoldingRangeKind::Region),
        }];

        assert_eq!(actual_foldings, expected_foldings);
    }

    #[tokio::test]
    async fn comment() {
        let actual_foldings = FeatureTester::new()
            .file("main.bib", "foo")
            .main("main.bib")
            .test_folding(BibtexDeclarationFoldingProvider)
            .await;

        assert!(actual_foldings.is_empty());
    }

    #[tokio::test]
    async fn entry_invalid() {
        let actual_foldings = FeatureTester::new()
            .file("main.bib", "@article{foo,")
            .main("main.bib")
            .test_folding(BibtexDeclarationFoldingProvider)
            .await;

        assert!(actual_foldings.is_empty());
    }

    #[tokio::test]
    async fn latex() {
        let actual_foldings = FeatureTester::new()
            .file("main.tex", "foo")
            .main("main.tex")
            .test_folding(BibtexDeclarationFoldingProvider)
            .await;

        assert!(actual_foldings.is_empty());
    }
}
