use crate::{
    feature::{FeatureProvider, FeatureRequest},
    protocol::{LocationLink, Position, TextDocumentPositionParams, Uri},
    syntax::{bibtex, SyntaxNode},
    workspace::DocumentContent,
};
use futures_boxed::boxed;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct BibtexStringDefinitionProvider;

impl FeatureProvider for BibtexStringDefinitionProvider {
    type Params = TextDocumentPositionParams;
    type Output = Vec<LocationLink>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        if let DocumentContent::Bibtex(tree) = &req.current().content {
            if let Some(reference) = Self::find_reference(tree, req.params.position) {
                return Self::find_definitions(&req.current().uri, tree, reference);
            }
        }
        Vec::new()
    }
}

impl BibtexStringDefinitionProvider {
    fn find_reference(tree: &bibtex::Tree, pos: Position) -> Option<&bibtex::Token> {
        let mut nodes = tree.find(pos);
        nodes.reverse();
        match (
            &tree.graph[nodes[0]],
            nodes.get(1).map(|node| &tree.graph[*node]),
        ) {
            (bibtex::Node::Word(word), Some(bibtex::Node::Field(_)))
            | (bibtex::Node::Word(word), Some(bibtex::Node::Concat(_))) => Some(&word.token),
            _ => None,
        }
    }

    fn find_definitions(
        uri: &Uri,
        tree: &bibtex::Tree,
        reference: &bibtex::Token,
    ) -> Vec<LocationLink> {
        let mut links = Vec::new();
        for node in tree.children(tree.root) {
            if let Some(string) = tree.as_string(node) {
                if let Some(name) = &string.name {
                    if name.text() == reference.text() {
                        links.push(LocationLink {
                            origin_selection_range: Some(reference.range()),
                            target_uri: uri.clone().into(),
                            target_range: string.range(),
                            target_selection_range: name.range(),
                        });
                    }
                }
            }
        }
        links
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
        let actual_items = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_position(BibtexStringDefinitionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_position(BibtexStringDefinitionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn simple() {
        let actual_links = FeatureTester::new()
            .file(
                "main.bib",
                indoc!(
                    r#"
                        @string{foo = {bar}}
                        @article{bar, author = foo}
                    "#
                ),
            )
            .main("main.bib")
            .position(1, 24)
            .test_position(BibtexStringDefinitionProvider)
            .await;

        let expected_links = vec![LocationLink {
            origin_selection_range: Some(Range::new_simple(1, 23, 1, 26)),
            target_uri: FeatureTester::uri("main.bib").into(),
            target_range: Range::new_simple(0, 0, 0, 20),
            target_selection_range: Range::new_simple(0, 8, 0, 11),
        }];

        assert_eq!(actual_links, expected_links);
    }

    #[tokio::test]
    async fn concat() {
        let actual_links = FeatureTester::new()
            .file(
                "main.bib",
                indoc!(
                    r#"
                        @string{foo = {bar}}
                        @article{bar, author = foo # "bar"}
                    "#
                ),
            )
            .main("main.bib")
            .position(1, 24)
            .test_position(BibtexStringDefinitionProvider)
            .await;

        let expected_links = vec![LocationLink {
            origin_selection_range: Some(Range::new_simple(1, 23, 1, 26)),
            target_uri: FeatureTester::uri("main.bib").into(),
            target_range: Range::new_simple(0, 0, 0, 20),
            target_selection_range: Range::new_simple(0, 8, 0, 11),
        }];

        assert_eq!(actual_links, expected_links);
    }

    #[tokio::test]
    async fn field() {
        let actual_links = FeatureTester::new()
            .file(
                "main.bib",
                indoc!(
                    r#"
                        @string{foo = {bar}}
                        @article{bar, author = foo}
                    "#
                ),
            )
            .main("main.bib")
            .position(1, 18)
            .test_position(BibtexStringDefinitionProvider)
            .await;

        assert!(actual_links.is_empty());
    }
}
