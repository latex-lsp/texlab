use crate::formatting::bibtex::{self, BibtexFormattingParams};
use crate::syntax::*;
use crate::workspace::*;
use futures_boxed::boxed;
use texlab_protocol::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BibtexStringReferenceHoverProvider;

impl FeatureProvider for BibtexStringReferenceHoverProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Hover>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        if let SyntaxTree::Bibtex(tree) = &request.document().tree {
            let reference = Self::find_reference(tree, request.params.position)?;
            for declaration in &tree.root.children {
                if let BibtexDeclaration::String(string) = &declaration {
                    let definition = Self::find_definition(string, reference);
                    if definition.is_some() {
                        return definition;
                    }
                }
            }
        }
        None
    }
}

impl BibtexStringReferenceHoverProvider {
    fn find_reference(tree: &BibtexSyntaxTree, position: Position) -> Option<&BibtexToken> {
        let mut results = tree.find(position);
        results.reverse();
        match (&results[0], results.get(1)) {
            (BibtexNode::Word(reference), Some(BibtexNode::Concat(_))) => Some(&reference.token),
            (BibtexNode::Word(reference), Some(BibtexNode::Field(_))) => Some(&reference.token),
            _ => None,
        }
    }

    fn find_definition(string: &BibtexString, reference: &BibtexToken) -> Option<Hover> {
        if string.name.as_ref()?.text() != reference.text() {
            return None;
        }

        let text =
            bibtex::format_content(string.value.as_ref()?, &BibtexFormattingParams::default());
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
    use texlab_protocol::RangeExt;

    #[test]
    fn test_inside_reference() {
        let hover = test_feature(
            BibtexStringReferenceHoverProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.bib",
                    "@string{foo = \"Foo\"}\n@string{bar = \"Bar\"}\n@article{baz, author = bar}",
                )],
                main_file: "foo.bib",
                position: Position::new(2, 24),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            hover,
            Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::PlainText,
                    value: "\"Bar\"".into(),
                }),
                range: Some(Range::new_simple(2, 23, 2, 26)),
            })
        );
    }

    #[test]
    fn test_outside_reference() {
        let hover = test_feature(
            BibtexStringReferenceHoverProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.bib",
                    "@string{foo = \"Foo\"}\n@string{bar = \"Bar\"}\n@article{baz, author = bar}",
                )],
                main_file: "foo.bib",
                position: Position::new(2, 20),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(hover, None);
    }
}
