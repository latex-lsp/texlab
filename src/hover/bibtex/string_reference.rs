use crate::{
    feature::{FeatureProvider, FeatureRequest},
    protocol::{
        BibtexFormattingOptions, Hover, HoverContents, MarkupContent, MarkupKind, Position,
        TextDocumentPositionParams,
    },
    syntax::{bibtex, SyntaxNode},
};
use futures_boxed::boxed;
use petgraph::graph::NodeIndex;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct BibtexStringReferenceHoverProvider;

impl FeatureProvider for BibtexStringReferenceHoverProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Hover>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let tree = req.current().content.as_bibtex()?;
        let reference = Self::find_reference(tree, req.params.position)?;
        for string_node in tree.children(tree.root) {
            let hover = Self::find_definition(tree, string_node, reference);
            if hover.is_some() {
                return hover;
            }
        }
        None
    }
}

impl BibtexStringReferenceHoverProvider {
    fn find_reference(tree: &bibtex::Tree, pos: Position) -> Option<&bibtex::Token> {
        let mut results = tree.find(pos);
        results.reverse();
        match (
            &tree.graph[results[0]],
            results.get(1).map(|node| &tree.graph[*node]),
        ) {
            (bibtex::Node::Word(reference), Some(bibtex::Node::Concat(_))) => {
                Some(&reference.token)
            }
            (bibtex::Node::Word(reference), Some(bibtex::Node::Field(_))) => Some(&reference.token),
            _ => None,
        }
    }

    fn find_definition(
        tree: &bibtex::Tree,
        string_node: NodeIndex,
        reference: &bibtex::Token,
    ) -> Option<Hover> {
        let string = tree.as_string(string_node)?;
        if string.name.as_ref()?.text() != reference.text() {
            return None;
        }

        let options = BibtexFormattingOptions { line_length: None };
        let text = bibtex::format(
            tree,
            tree.children(string_node).next()?,
            bibtex::FormattingParams {
                tab_size: 4,
                insert_spaces: true,
                options: &options,
            },
        );
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::PlainText,
                value: text,
            }),
            range: Some(reference.range()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        feature::FeatureTester,
        protocol::{Range, RangeExt},
    };
    use indoc::indoc;

    #[tokio::test]
    async fn empty_latex_document() {
        let actual_hover = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_position(BibtexStringReferenceHoverProvider)
            .await;

        assert_eq!(actual_hover, None);
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_hover = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_position(BibtexStringReferenceHoverProvider)
            .await;

        assert_eq!(actual_hover, None);
    }

    #[tokio::test]
    async fn inside_reference() {
        let actual_hover = FeatureTester::new()
            .file(
                "main.bib",
                indoc!(
                    r#"
                        @string{foo = "Foo"}
                        @string{bar = "Bar"}
                        @article{baz, author = bar}
                    "#
                ),
            )
            .main("main.bib")
            .position(2, 24)
            .test_position(BibtexStringReferenceHoverProvider)
            .await
            .unwrap();

        let expected_hover = Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::PlainText,
                value: "\"Bar\"".into(),
            }),
            range: Some(Range::new_simple(2, 23, 2, 26)),
        };

        assert_eq!(actual_hover, expected_hover);
    }

    #[tokio::test]
    async fn outside_reference() {
        let actual_hover = FeatureTester::new()
            .file(
                "main.bib",
                indoc!(
                    r#"
                        @string{foo = "Foo"}
                        @string{bar = "Bar"}
                        @article{baz, author = bar}
                    "#
                ),
            )
            .main("main.bib")
            .position(2, 20)
            .test_position(BibtexStringReferenceHoverProvider)
            .await;

        assert_eq!(actual_hover, None);
    }
}
