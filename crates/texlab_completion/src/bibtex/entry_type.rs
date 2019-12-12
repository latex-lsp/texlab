use crate::factory;
use texlab_workspace::*;
use futures_boxed::boxed;
use texlab_protocol::*;
use texlab_syntax::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BibtexEntryTypeCompletionProvider;

impl FeatureProvider for BibtexEntryTypeCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        if let SyntaxTree::Bibtex(tree) = &request.document().tree {
            let position = request.params.text_document_position.position;
            for declaration in &tree.root.children {
                match declaration {
                    BibtexDeclaration::Preamble(preamble) => {
                        if contains(&preamble.ty, position) {
                            return make_items(request, preamble.ty.range());
                        }
                    }
                    BibtexDeclaration::String(string) => {
                        if contains(&string.ty, position) {
                            return make_items(request, string.ty.range());
                        }
                    }
                    BibtexDeclaration::Entry(entry) => {
                        if contains(&entry.ty, position) {
                            return make_items(request, entry.ty.range());
                        }
                    }
                    BibtexDeclaration::Comment(_) => {}
                }
            }
        }
        Vec::new()
    }
}

fn contains(ty: &BibtexToken, position: Position) -> bool {
    ty.range().contains(position) && ty.start().character != position.character
}

fn make_items(request: &FeatureRequest<CompletionParams>, mut range: Range) -> Vec<CompletionItem> {
    range.start.character += 1;
    let mut items = Vec::new();
    for ty in &LANGUAGE_DATA.entry_types {
        let text_edit = TextEdit::new(range, (&ty.name).into());
        let item = factory::entry_type(request, ty, text_edit);
        items.push(item);
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_before_at_sign() {
        let items = test_feature(
            BibtexEntryTypeCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@")],
                main_file: "foo.bib",
                position: Position::new(0, 0),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }

    #[test]
    fn test_after_at_sign() {
        let items = test_feature(
            BibtexEntryTypeCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@")],
                main_file: "foo.bib",
                position: Position::new(0, 1),
                ..FeatureSpec::default()
            },
        );
        assert!(!items.is_empty());
        assert_eq!(
            items[0].text_edit.as_ref().map(|edit| edit.range),
            Some(Range::new_simple(0, 1, 0, 1))
        );
    }

    #[test]
    fn test_inside_entry_type() {
        let items = test_feature(
            BibtexEntryTypeCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@foo")],
                main_file: "foo.bib",
                position: Position::new(0, 2),
                ..FeatureSpec::default()
            },
        );
        assert!(!items.is_empty());
        assert_eq!(
            items[0].text_edit.as_ref().map(|edit| edit.range),
            Some(Range::new_simple(0, 1, 0, 4))
        );
    }

    #[test]
    fn test_inside_entry_key() {
        let items = test_feature(
            BibtexEntryTypeCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo,}")],
                main_file: "foo.bib",
                position: Position::new(0, 11),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }

    #[test]
    fn test_inside_comments() {
        let items = test_feature(
            BibtexEntryTypeCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "foo")],
                main_file: "foo.bib",
                position: Position::new(0, 2),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }

    #[test]
    fn test_latex() {
        let items = test_feature(
            BibtexEntryTypeCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "@")],
                main_file: "foo.tex",
                position: Position::new(0, 1),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }
}
