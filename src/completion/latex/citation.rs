use super::combinators::{self, Parameter};
use crate::completion::factory;
use crate::syntax::*;
use crate::workspace::*;
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams, TextEdit};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexCitationCompletionProvider;

impl FeatureProvider for LatexCitationCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let parameters = LANGUAGE_DATA
            .citation_commands
            .iter()
            .map(|cmd| Parameter::new(&cmd.name, cmd.index));

        combinators::argument(request, parameters, async move |context| {
            let mut items = Vec::new();
            for document in request.related_documents() {
                if let SyntaxTree::Bibtex(tree) = &document.tree {
                    for entry in &tree.entries() {
                        if !entry.is_comment() {
                            if let Some(key) = &entry.key {
                                let key = key.text().to_owned();
                                let text_edit = TextEdit::new(context.range, key.clone().into());
                                let item = factory::citation(request, entry, key, text_edit);
                                items.push(item);
                            }
                        }
                    }
                }
            }
            items
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{Position, Range};

    #[test]
    fn test_empty() {
        let items = test_feature(
            LatexCitationCompletionProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\addbibresource{bar.bib}\n\\cite{}"),
                    FeatureSpec::file("bar.bib", "@article{foo,}"),
                    FeatureSpec::file("baz.bib", "@article{bar,}"),
                ],
                main_file: "foo.tex",
                position: Position::new(1, 6),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "foo");
        assert_eq!(
            items[0].text_edit.as_ref().map(|edit| edit.range),
            Some(Range::new_simple(1, 6, 1, 6))
        );
    }

    #[test]
    fn test_single_key() {
        let items = test_feature(
            LatexCitationCompletionProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\addbibresource{bar.bib}\n\\cite{foo}"),
                    FeatureSpec::file("bar.bib", "@article{foo,}"),
                    FeatureSpec::file("baz.bib", "@article{bar,}"),
                ],
                main_file: "foo.tex",
                position: Position::new(1, 6),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "foo");
        assert_eq!(
            items[0].text_edit.as_ref().map(|edit| edit.range),
            Some(Range::new_simple(1, 6, 1, 9))
        );
    }

    #[test]
    fn test_second_key() {
        let items = test_feature(
            LatexCitationCompletionProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\addbibresource{bar.bib}\n\\cite{foo,}"),
                    FeatureSpec::file("bar.bib", "@article{foo,}"),
                    FeatureSpec::file("baz.bib", "@article{bar,}"),
                ],
                main_file: "foo.tex",
                position: Position::new(1, 10),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "foo");
        assert_eq!(
            items[0].text_edit.as_ref().map(|edit| edit.range),
            Some(Range::new_simple(1, 10, 1, 10))
        );
    }

    #[test]
    fn test_outside_cite() {
        let items = test_feature(
            LatexCitationCompletionProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\addbibresource{bar.bib}\n\\cite{}"),
                    FeatureSpec::file("bar.bib", "@article{foo,}"),
                    FeatureSpec::file("baz.bib", "@article{bar,}"),
                ],
                main_file: "foo.tex",
                position: Position::new(1, 7),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }
}
