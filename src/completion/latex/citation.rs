use crate::completion::factory;
use crate::completion::latex::combinators::{self, ArgumentLocation};
use crate::data::language::language_data;
use crate::feature::{FeatureProvider, FeatureRequest};
use crate::syntax::SyntaxTree;
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexCitationCompletionProvider;

impl FeatureProvider for LatexCitationCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<Arc<CompletionItem>>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let locations = language_data()
            .citation_commands
            .iter()
            .map(|cmd| ArgumentLocation::new(&cmd.name, cmd.index));

        combinators::argument(request, locations, async move |_| {
            let mut items = Vec::new();
            for document in request.related_documents() {
                if let SyntaxTree::Bibtex(tree) = &document.tree {
                    for entry in &tree.entries() {
                        if !entry.is_comment() {
                            if let Some(key) = &entry.key {
                                items.push(Arc::new(factory::create_citation(entry, key.text())));
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
    use crate::feature::{test_feature, FeatureSpec};
    use lsp_types::Position;

    #[test]
    fn test_inside_cite() {
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
