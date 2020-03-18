use crate::{
    completion::factory,
    feature::{FeatureProvider, FeatureRequest},
    protocol::{CompletionItem, CompletionParams, Position, Range, RangeExt, TextEdit},
    syntax::{bibtex, SyntaxNode, LANGUAGE_DATA},
    workspace::DocumentContent,
};
use futures_boxed::boxed;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct BibtexEntryTypeCompletionProvider;

impl FeatureProvider for BibtexEntryTypeCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        if let DocumentContent::Bibtex(tree) = &req.current().content {
            let pos = req.params.text_document_position.position;
            for decl in tree.children(tree.root) {
                match &tree.graph[decl] {
                    bibtex::Node::Preamble(preamble) => {
                        if Self::contains(&preamble.ty, pos) {
                            return Self::make_items(req, preamble.ty.range());
                        }
                    }
                    bibtex::Node::String(string) => {
                        if Self::contains(&string.ty, pos) {
                            return Self::make_items(req, string.ty.range());
                        }
                    }
                    bibtex::Node::Entry(entry) => {
                        if Self::contains(&entry.ty, pos) {
                            return Self::make_items(req, entry.ty.range());
                        }
                    }
                    _ => {}
                }
            }
        }
        Vec::new()
    }
}

impl BibtexEntryTypeCompletionProvider {
    fn contains(ty: &bibtex::Token, pos: Position) -> bool {
        ty.range().contains(pos) && ty.start().character != pos.character
    }

    fn make_items(req: &FeatureRequest<CompletionParams>, mut range: Range) -> Vec<CompletionItem> {
        range.start.character += 1;
        let mut items = Vec::new();
        for ty in &LANGUAGE_DATA.entry_types {
            let text_edit = TextEdit::new(range, (&ty.name).into());
            let item = factory::entry_type(req, ty, text_edit);
            items.push(item);
        }
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;

    #[tokio::test]
    async fn empty_latex_document() {
        let actual_items = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion(BibtexEntryTypeCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion(BibtexEntryTypeCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn before_at_sign() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "@")
            .main("main.bib")
            .position(0, 0)
            .test_completion(BibtexEntryTypeCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn after_at_sign() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "@")
            .main("main.bib")
            .position(0, 1)
            .test_completion(BibtexEntryTypeCompletionProvider)
            .await;

        assert!(!actual_items.is_empty());
        assert_eq!(
            actual_items[0]
                .text_edit
                .as_ref()
                .map(|edit| edit.range)
                .unwrap(),
            Range::new_simple(0, 1, 0, 1)
        );
    }

    #[tokio::test]
    async fn inside_entry_type() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "@foo")
            .main("main.bib")
            .position(0, 2)
            .test_completion(BibtexEntryTypeCompletionProvider)
            .await;

        assert!(!actual_items.is_empty());
        assert_eq!(
            actual_items[0]
                .text_edit
                .as_ref()
                .map(|edit| edit.range)
                .unwrap(),
            Range::new_simple(0, 1, 0, 4)
        );
    }

    #[tokio::test]
    async fn inside_entry_key() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "@article{foo,}")
            .main("main.bib")
            .position(0, 11)
            .test_completion(BibtexEntryTypeCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_comments() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "foo")
            .main("main.bib")
            .position(0, 2)
            .test_completion(BibtexEntryTypeCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }
}
