use crate::completion::factory;
use crate::syntax::*;
use crate::workspace::*;
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams, Range, TextEdit};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct BibtexFieldNameCompletionProvider;

impl FeatureProvider for BibtexFieldNameCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        if let SyntaxTree::Bibtex(tree) = &request.document().tree {
            let position = request.params.position;
            match tree.find(position).last() {
                Some(BibtexNode::Field(field)) => {
                    if field.name.range().contains(position) {
                        return make_items(request, field.name.range());
                    }
                }
                Some(BibtexNode::Entry(entry)) => {
                    if !entry.is_comment() && !entry.ty.range().contains(position) {
                        let edit_range = Range::new(position, position);
                        if let Some(key) = &entry.key {
                            if !key.range().contains(position) {
                                return make_items(request, edit_range);
                            }
                        } else {
                            return make_items(request, edit_range);
                        }
                    }
                }
                _ => {}
            }
        }
        Vec::new()
    }
}

fn make_items(
    request: &FeatureRequest<CompletionParams>,
    edit_range: Range,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    for field in &LANGUAGE_DATA.fields {
        let text_edit = TextEdit::new(edit_range, (&field.name).into());
        let item = factory::field_name(request, field, text_edit);
        items.push(item);
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::Position;

    #[test]
    fn test_inside_first_field() {
        let items = test_feature(
            BibtexFieldNameCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo,\nbar}")],
                main_file: "foo.bib",
                position: Position::new(1, 1),
                ..FeatureSpec::default()
            },
        );
        assert!(!items.is_empty());
        assert_eq!(
            items[0].text_edit.as_ref().map(|edit| edit.range),
            Some(Range::new_simple(1, 0, 1, 3))
        );
    }

    #[test]
    fn test_inside_second_field() {
        let items = test_feature(
            BibtexFieldNameCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.bib",
                    "@article{foo, bar = {baz}, qux}",
                )],
                main_file: "foo.bib",
                position: Position::new(0, 27),
                ..FeatureSpec::default()
            },
        );
        assert!(!items.is_empty());
        assert_eq!(
            items[0].text_edit.as_ref().map(|edit| edit.range),
            Some(Range::new_simple(0, 27, 0, 30))
        );
    }

    #[test]
    fn test_inside_entry() {
        let items = test_feature(
            BibtexFieldNameCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, \n}")],
                main_file: "foo.bib",
                position: Position::new(1, 0),
                ..FeatureSpec::default()
            },
        );
        assert!(!items.is_empty());
        assert_eq!(
            items[0].text_edit.as_ref().map(|edit| edit.range),
            Some(Range::new_simple(1, 0, 1, 0))
        );
    }

    #[test]
    fn test_inside_content() {
        let items = test_feature(
            BibtexFieldNameCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo,\nbar = {baz}}")],
                main_file: "foo.bib",
                position: Position::new(1, 7),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }

    #[test]
    fn test_inside_entry_type() {
        let items = test_feature(
            BibtexFieldNameCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo,}")],
                main_file: "foo.bib",
                position: Position::new(0, 3),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }

    // TODO: Improve behavior of this provider
    //
    //    #[test]
    //    fn test_after_equals_sign() {
    //        let items = test_feature(
    //            BibtexFieldNameCompletionProvider,
    //            FeatureSpec {
    //                files: vec![FeatureSpec::file("foo.bib", "@article{foo, bar = \n}")],
    //                main_file: "foo.bib",
    //                position: Position::new(1, 0),
    //                ..FeatureSpec::default()
    //            },
    //        );
    //        assert!(items.is_empty());
    //    }

    #[test]
    fn test_inside_latex() {
        let items = test_feature(
            BibtexFieldNameCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "@article{foo,}")],
                main_file: "foo.tex",
                position: Position::new(0, 3),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }
}
