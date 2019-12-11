use texlab_protocol::RangeExt;
use crate::syntax::*;
use crate::workspace::*;
use futures_boxed::boxed;
use lsp_types::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BibtexEntryTypeHoverProvider;

impl FeatureProvider for BibtexEntryTypeHoverProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Hover>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<TextDocumentPositionParams>,
    ) -> Option<Hover> {
        if let SyntaxTree::Bibtex(tree) = &request.document().tree {
            for entry in tree.entries() {
                if entry.ty.range().contains(request.params.position) {
                    let ty = &entry.ty.text()[1..];
                    if let Some(documentation) = LANGUAGE_DATA.entry_type_documentation(ty) {
                        return Some(Hover {
                            contents: HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: documentation.into(),
                            }),
                            range: None,
                        });
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::Position;

    #[test]
    fn test_known_entry_type() {
        let hover = test_feature(
            BibtexEntryTypeHoverProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo,}")],
                main_file: "foo.bib",
                position: Position::new(0, 3),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            hover,
            Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: LANGUAGE_DATA
                        .entry_type_documentation("article")
                        .unwrap()
                        .into(),
                }),
                range: None,
            })
        );
    }

    #[test]
    fn test_unknown_entry_type() {
        let hover = test_feature(
            BibtexEntryTypeHoverProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@foo{bar,}")],
                main_file: "foo.bib",
                position: Position::new(0, 3),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(hover, None);
    }

    #[test]
    fn test_entry_key() {
        let hover = test_feature(
            BibtexEntryTypeHoverProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo,}")],
                main_file: "foo.bib",
                position: Position::new(0, 11),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(hover, None);
    }

    #[test]
    fn test_latex() {
        let hover = test_feature(
            BibtexEntryTypeHoverProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\foo")],
                main_file: "foo.tex",
                position: Position::new(0, 3),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(hover, None);
    }
}
