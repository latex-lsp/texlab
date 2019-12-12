use texlab_workspace::*;
use futures_boxed::boxed;
use texlab_protocol::*;
use texlab_syntax::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BibtexStringDefinitionProvider;

impl FeatureProvider for BibtexStringDefinitionProvider {
    type Params = TextDocumentPositionParams;
    type Output = Vec<LocationLink>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        if let SyntaxTree::Bibtex(tree) = &request.document().tree {
            if let Some(reference) = Self::find_reference(tree, request.params.position) {
                return Self::find_definitions(&request.view.document.uri, tree, reference);
            }
        }
        Vec::new()
    }
}

impl BibtexStringDefinitionProvider {
    fn find_reference(tree: &BibtexSyntaxTree, position: Position) -> Option<&BibtexToken> {
        let mut nodes = tree.find(position);
        nodes.reverse();
        match (&nodes[0], &nodes.get(1)) {
            (BibtexNode::Word(word), Some(BibtexNode::Field(_)))
            | (BibtexNode::Word(word), Some(BibtexNode::Concat(_))) => Some(&word.token),
            _ => None,
        }
    }

    fn find_definitions(
        uri: &Uri,
        tree: &BibtexSyntaxTree,
        reference: &BibtexToken,
    ) -> Vec<LocationLink> {
        let mut links = Vec::new();
        for string in tree.strings() {
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
        links
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use texlab_protocol::RangeExt;
    use texlab_protocol::{Position, Range};

    #[test]
    fn test_simple() {
        let links = test_feature(
            BibtexStringDefinitionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.bib",
                    "@string{foo = {bar}}\n@article{bar, author = foo}",
                )],
                main_file: "foo.bib",
                position: Position::new(1, 24),
                ..FeatureSpec::default()
            },
        );

        assert_eq!(
            links,
            vec![LocationLink {
                origin_selection_range: Some(Range::new_simple(1, 23, 1, 26)),
                target_uri: FeatureSpec::uri("foo.bib"),
                target_range: Range::new_simple(0, 0, 0, 20),
                target_selection_range: Range::new_simple(0, 8, 0, 11)
            }]
        );
    }

    #[test]
    fn test_concat() {
        let links = test_feature(
            BibtexStringDefinitionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.bib",
                    "@string{foo = {bar}}\n@article{bar, author = foo # \"bar\"}",
                )],
                main_file: "foo.bib",
                position: Position::new(1, 24),
                ..FeatureSpec::default()
            },
        );

        assert_eq!(
            links,
            vec![LocationLink {
                origin_selection_range: Some(Range::new_simple(1, 23, 1, 26)),
                target_uri: FeatureSpec::uri("foo.bib"),
                target_range: Range::new_simple(0, 0, 0, 20),
                target_selection_range: Range::new_simple(0, 8, 0, 11)
            }]
        );
    }

    #[test]
    fn test_field() {
        let links = test_feature(
            BibtexStringDefinitionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.bib",
                    "@string{foo = {bar}}\n@article{bar, author = foo}",
                )],
                main_file: "foo.bib",
                position: Position::new(1, 18),
                ..FeatureSpec::default()
            },
        );

        assert!(links.is_empty());
    }
}
