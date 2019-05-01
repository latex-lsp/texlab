use crate::feature::FeatureRequest;
use crate::metadata::bibtex_field;
use crate::syntax::bibtex::ast::{BibtexField, BibtexVisitor};
use crate::syntax::bibtex::finder::{BibtexFinder, BibtexNode};
use crate::syntax::text::SyntaxNode;
use crate::workspace::SyntaxTree;
use lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, TextDocumentPositionParams};

pub struct BibtexFieldHoverProvider;

impl BibtexFieldHoverProvider {
    pub async fn execute(request: &FeatureRequest<TextDocumentPositionParams>) -> Option<Hover> {
        if let SyntaxTree::Bibtex(tree) = &request.document.tree {
            let mut finder = BibtexFinder::new(request.params.position);
            finder.visit_root(&tree.root);
            for node in finder.results {
                if let BibtexNode::Field(field) = node {
                    let documentation = bibtex_field::get_documentation(field.name.text())?;
                    return Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: documentation.to_owned(),
                        }),
                        range: None,
                    });
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;
    use crate::workspace::WorkspaceBuilder;
    use futures::executor::block_on;

    #[test]
    fn test_known_field() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.bib", "@article{foo, author = bar}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 15, "").into();

        let actual = block_on(BibtexFieldHoverProvider::execute(&request));

        let expected = Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: bibtex_field::get_documentation("author")
                    .unwrap()
                    .to_owned(),
            }),
            range: None,
        };
        assert_eq!(Some(expected), actual);
    }

    #[test]
    fn test_unknown_field() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.bib", "@article{foo, bar = baz}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 15, "").into();

        let actual = block_on(BibtexFieldHoverProvider::execute(&request));

        assert_eq!(None, actual);
    }

    #[test]
    fn test_entry_key() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.bib", "@article{foo, bar = baz}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 11, "").into();

        let actual = block_on(BibtexFieldHoverProvider::execute(&request));

        assert_eq!(None, actual);
    }

    #[test]
    fn test_latex() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "\\foo");
        let request = FeatureTester::new(builder.workspace, uri, 0, 1, "").into();

        let actual = block_on(BibtexFieldHoverProvider::execute(&request));

        assert_eq!(None, actual);
    }
}
