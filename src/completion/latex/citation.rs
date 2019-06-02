use crate::completion::factory;
use crate::completion::latex::combinators::LatexCombinators;
use crate::feature::{FeatureProvider, FeatureRequest};
use crate::syntax::latex::CITATION_COMMANDS;
use crate::syntax::SyntaxTree;
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexCitationCompletionProvider;

impl FeatureProvider for LatexCitationCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<CompletionParams>,
    ) -> Vec<CompletionItem> {
        LatexCombinators::argument(request, CITATION_COMMANDS, 0, async move |_| {
            let mut items = Vec::new();
            for document in &request.related_documents {
                if let SyntaxTree::Bibtex(tree) = &document.tree {
                    for entry in &tree.entries() {
                        if !entry.is_comment() {
                            if let Some(key) = &entry.key {
                                items.push(factory::create_citation(entry, key.text()));
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
