use crate::workspace::*;
use futures_boxed::boxed;
use texlab_protocol::RangeExt;
use texlab_protocol::{Location, Position, ReferenceParams, Url};
use texlab_syntax::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BibtexStringReferenceProvider;

impl FeatureProvider for BibtexStringReferenceProvider {
    type Params = ReferenceParams;
    type Output = Vec<Location>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<ReferenceParams>) -> Vec<Location> {
        let mut references = Vec::new();
        if let SyntaxTree::Bibtex(tree) = &request.document().tree {
            if let Some(name) =
                Self::find_name(tree, request.params.text_document_position.position)
            {
                let uri: Url = request.document().uri.clone().into();
                if request.params.context.include_declaration {
                    for string in tree.strings() {
                        if let Some(string_name) = &string.name {
                            if string_name.text() == name.text() {
                                references.push(Location::new(uri.clone(), string_name.range()));
                            }
                        }
                    }
                }

                let mut visitor = BibtexStringReferenceVisitor::default();
                visitor.visit_root(&tree.root);
                visitor
                    .references
                    .into_iter()
                    .filter(|reference| reference.text() == name.text())
                    .map(|reference| Location::new(uri.clone(), reference.range()))
                    .for_each(|reference| references.push(reference));
            }
        }
        references
    }
}

impl BibtexStringReferenceProvider {
    fn find_name(tree: &BibtexSyntaxTree, position: Position) -> Option<&BibtexToken> {
        let mut nodes = tree.find(position);
        nodes.reverse();
        match (&nodes[0], nodes.get(1)) {
            (BibtexNode::Word(word), Some(BibtexNode::Field(_)))
            | (BibtexNode::Word(word), Some(BibtexNode::Concat(_))) => Some(&word.token),
            (BibtexNode::String(string), _) => string
                .name
                .as_ref()
                .filter(|name| name.range().contains(position)),
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
struct BibtexStringReferenceVisitor<'a> {
    references: Vec<&'a BibtexToken>,
}

impl<'a> BibtexVisitor<'a> for BibtexStringReferenceVisitor<'a> {
    fn visit_root(&mut self, root: &'a BibtexRoot) {
        BibtexWalker::walk_root(self, root);
    }

    fn visit_comment(&mut self, _comment: &'a BibtexComment) {}

    fn visit_preamble(&mut self, preamble: &'a BibtexPreamble) {
        BibtexWalker::walk_preamble(self, preamble);
    }

    fn visit_string(&mut self, string: &'a BibtexString) {
        BibtexWalker::walk_string(self, string);
    }

    fn visit_entry(&mut self, entry: &'a BibtexEntry) {
        BibtexWalker::walk_entry(self, entry);
    }

    fn visit_field(&mut self, field: &'a BibtexField) {
        if let Some(BibtexContent::Word(word)) = &field.content {
            self.references.push(&word.token);
        }

        BibtexWalker::walk_field(self, field);
    }

    fn visit_word(&mut self, _word: &'a BibtexWord) {}

    fn visit_command(&mut self, _command: &'a BibtexCommand) {}

    fn visit_quoted_content(&mut self, content: &'a BibtexQuotedContent) {
        BibtexWalker::walk_quoted_content(self, content);
    }

    fn visit_braced_content(&mut self, content: &'a BibtexBracedContent) {
        BibtexWalker::walk_braced_content(self, content);
    }

    fn visit_concat(&mut self, concat: &'a BibtexConcat) {
        if let BibtexContent::Word(word) = &concat.left {
            self.references.push(&word.token);
        }

        if let Some(BibtexContent::Word(word)) = &concat.right {
            self.references.push(&word.token);
        }

        BibtexWalker::walk_concat(self, concat);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use texlab_protocol::RangeExt;
    use texlab_protocol::{Position, Range};

    #[test]
    fn test_definition() {
        let references = test_feature(
            BibtexStringReferenceProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.bib",
                    "@string{foo = {Foo}}\n@string{bar = {Bar}}\n@article{baz, author = foo}",
                )],
                main_file: "foo.bib",
                position: Position::new(2, 24),
                include_declaration: false,
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            references,
            vec![Location::new(
                FeatureSpec::uri("foo.bib"),
                Range::new_simple(2, 23, 2, 26)
            )]
        );
    }

    #[test]
    fn test_definition_include_declaration() {
        let references = test_feature(
            BibtexStringReferenceProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.bib",
                    "@string{foo = {Foo}}\n@string{bar = {Bar}}\n@article{baz, author = foo}",
                )],
                main_file: "foo.bib",
                position: Position::new(2, 24),
                include_declaration: true,
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            references,
            vec![
                Location::new(FeatureSpec::uri("foo.bib"), Range::new_simple(0, 8, 0, 11)),
                Location::new(FeatureSpec::uri("foo.bib"), Range::new_simple(2, 23, 2, 26))
            ]
        );
    }

    #[test]
    fn test_reference() {
        let references = test_feature(
            BibtexStringReferenceProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.bib",
                    "@string{foo = {Foo}}\n@string{bar = {Bar}}\n@article{baz, author = foo}",
                )],
                main_file: "foo.bib",
                position: Position::new(0, 10),
                include_declaration: false,
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            references,
            vec![Location::new(
                FeatureSpec::uri("foo.bib"),
                Range::new_simple(2, 23, 2, 26)
            )]
        );
    }

    #[test]
    fn test_reference_include_declaration() {
        let references = test_feature(
            BibtexStringReferenceProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.bib",
                    "@string{foo = {Foo}}\n@string{bar = {Bar}}\n@article{baz, author = foo}",
                )],
                main_file: "foo.bib",
                position: Position::new(0, 10),
                include_declaration: true,
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            references,
            vec![
                Location::new(FeatureSpec::uri("foo.bib"), Range::new_simple(0, 8, 0, 11)),
                Location::new(FeatureSpec::uri("foo.bib"), Range::new_simple(2, 23, 2, 26))
            ]
        );
    }

    #[test]
    fn test_empty() {
        let references = test_feature(
            BibtexStringReferenceProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "")],
                main_file: "foo.bib",
                position: Position::new(0, 0),
                include_declaration: false,
                ..FeatureSpec::default()
            },
        );
        assert!(references.is_empty());
    }
}
