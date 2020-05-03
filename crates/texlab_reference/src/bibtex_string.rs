use async_trait::async_trait;
use petgraph::graph::NodeIndex;
use texlab_feature::{DocumentContent, FeatureProvider, FeatureRequest};
use texlab_protocol::{Location, Position, RangeExt, ReferenceParams, Url};
use texlab_syntax::{
    bibtex::{self, Visitor},
    SyntaxNode,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct BibtexStringReferenceProvider;

#[async_trait]
impl FeatureProvider for BibtexStringReferenceProvider {
    type Params = ReferenceParams;
    type Output = Vec<Location>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut refs = Vec::new();
        if let DocumentContent::Bibtex(tree) = &req.current().content {
            if let Some(name) = Self::find_name(tree, req.params.text_document_position.position) {
                let uri: Url = req.current().uri.clone().into();
                if req.params.context.include_declaration {
                    tree.children(tree.root)
                        .filter_map(|node| tree.as_string(node))
                        .filter_map(|string| string.name.as_ref())
                        .filter(|string| string.text() == name.text())
                        .for_each(|string| refs.push(Location::new(uri.clone(), string.range())));
                }

                let mut visitor = BibtexStringReferenceVisitor::default();
                visitor.visit(tree, tree.root);
                visitor
                    .refs
                    .into_iter()
                    .filter(|reference| reference.text() == name.text())
                    .for_each(|reference| refs.push(Location::new(uri.clone(), reference.range())));
            }
        }
        refs
    }
}

impl BibtexStringReferenceProvider {
    fn find_name(tree: &bibtex::Tree, pos: Position) -> Option<&bibtex::Token> {
        let mut nodes = tree.find(pos);
        nodes.reverse();
        let node0 = &tree.graph[nodes[0]];
        let node1 = nodes.get(1).map(|node| &tree.graph[*node]);
        match (node0, node1) {
            (bibtex::Node::Word(word), Some(bibtex::Node::Field(_)))
            | (bibtex::Node::Word(word), Some(bibtex::Node::Concat(_))) => Some(&word.token),
            (bibtex::Node::String(string), _) => string
                .name
                .as_ref()
                .filter(|name| name.range().contains(pos)),
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
pub struct BibtexStringReferenceVisitor<'a> {
    refs: Vec<&'a bibtex::Token>,
}

impl<'a> bibtex::Visitor<'a> for BibtexStringReferenceVisitor<'a> {
    fn visit(&mut self, tree: &'a bibtex::Tree, node: NodeIndex) {
        match &tree.graph[node] {
            bibtex::Node::Root(_)
            | bibtex::Node::Comment(_)
            | bibtex::Node::Preamble(_)
            | bibtex::Node::String(_)
            | bibtex::Node::Entry(_)
            | bibtex::Node::Word(_)
            | bibtex::Node::Command(_)
            | bibtex::Node::QuotedContent(_)
            | bibtex::Node::BracedContent(_) => (),
            bibtex::Node::Field(_) => {
                if let Some(word) = tree
                    .children(node)
                    .next()
                    .and_then(|content| tree.as_word(content))
                {
                    self.refs.push(&word.token);
                }
            }
            bibtex::Node::Concat(_) => {
                let mut children = tree.children(node);
                if let Some(word) = children.next().and_then(|left| tree.as_word(left)) {
                    self.refs.push(&word.token);
                }

                if let Some(word) = children.next().and_then(|right| tree.as_word(right)) {
                    self.refs.push(&word.token);
                }
            }
        }
        tree.walk(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use texlab_feature::FeatureTester;
    use texlab_protocol::Range;

    #[tokio::test]
    async fn definition() {
        let actual_refs = FeatureTester::new()
            .file(
                "main.bib",
                indoc!(
                    r#"
                        @string{foo = {Foo}}
                        @string{bar = {Bar}}
                        @article{baz, author = foo}
                    "#
                ),
            )
            .main("main.bib")
            .position(2, 24)
            .test_reference(BibtexStringReferenceProvider)
            .await;

        let expected_refs = vec![Location::new(
            FeatureTester::uri("main.bib").into(),
            Range::new_simple(2, 23, 2, 26),
        )];

        assert_eq!(actual_refs, expected_refs);
    }

    #[tokio::test]
    async fn definition_include_declaration() {
        let actual_refs = FeatureTester::new()
            .file(
                "main.bib",
                indoc!(
                    r#"
                        @string{foo = {Foo}}
                        @string{bar = {Bar}}
                        @article{baz, author = foo}
                    "#
                ),
            )
            .main("main.bib")
            .position(2, 24)
            .include_declaration()
            .test_reference(BibtexStringReferenceProvider)
            .await;

        let expected_refs = vec![
            Location::new(
                FeatureTester::uri("main.bib").into(),
                Range::new_simple(0, 8, 0, 11),
            ),
            Location::new(
                FeatureTester::uri("main.bib").into(),
                Range::new_simple(2, 23, 2, 26),
            ),
        ];

        assert_eq!(actual_refs, expected_refs);
    }

    #[tokio::test]
    async fn reference() {
        let actual_refs = FeatureTester::new()
            .file(
                "main.bib",
                indoc!(
                    r#"
                        @string{foo = {Foo}}
                        @string{bar = {Bar}}
                        @article{baz, author = foo}
                    "#
                ),
            )
            .main("main.bib")
            .position(0, 10)
            .test_reference(BibtexStringReferenceProvider)
            .await;

        let expected_refs = vec![Location::new(
            FeatureTester::uri("main.bib").into(),
            Range::new_simple(2, 23, 2, 26),
        )];

        assert_eq!(actual_refs, expected_refs);
    }

    #[tokio::test]
    async fn reference_include_declaration() {
        let actual_refs = FeatureTester::new()
            .file(
                "main.bib",
                indoc!(
                    r#"
                        @string{foo = {Foo}}
                        @string{bar = {Bar}}
                        @article{baz, author = foo}
                    "#
                ),
            )
            .main("main.bib")
            .position(0, 10)
            .include_declaration()
            .test_reference(BibtexStringReferenceProvider)
            .await;

        let expected_refs = vec![
            Location::new(
                FeatureTester::uri("main.bib").into(),
                Range::new_simple(0, 8, 0, 11),
            ),
            Location::new(
                FeatureTester::uri("main.bib").into(),
                Range::new_simple(2, 23, 2, 26),
            ),
        ];

        assert_eq!(actual_refs, expected_refs);
    }

    #[tokio::test]
    async fn empty_latex_document() {
        let actual_refs = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_reference(BibtexStringReferenceProvider)
            .await;

        assert!(actual_refs.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_refs = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_reference(BibtexStringReferenceProvider)
            .await;

        assert!(actual_refs.is_empty());
    }
}
