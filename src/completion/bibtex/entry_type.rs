use crate::{
    completion::types::{Item, ItemData},
    feature::FeatureRequest,
    protocol::{CompletionParams, Position, Range, RangeExt},
    syntax::{bibtex, SyntaxNode, LANGUAGE_DATA},
    workspace::DocumentContent,
};

pub async fn complete_bibtex_entry_types<'a>(
    req: &'a FeatureRequest<CompletionParams>,
    items: &mut Vec<Item<'a>>,
) {
    if let DocumentContent::Bibtex(tree) = &req.current().content {
        let pos = req.params.text_document_position.position;
        for decl in tree.children(tree.root) {
            match &tree.graph[decl] {
                bibtex::Node::Preamble(preamble) => {
                    if contains(&preamble.ty, pos) {
                        make_items(items, preamble.ty.range());
                        return;
                    }
                }
                bibtex::Node::String(string) => {
                    if contains(&string.ty, pos) {
                        make_items(items, string.ty.range());
                        return;
                    }
                }
                bibtex::Node::Entry(entry) => {
                    if contains(&entry.ty, pos) {
                        make_items(items, entry.ty.range());
                        return;
                    }
                }
                _ => {}
            }
        }
    }
}

fn contains(ty: &bibtex::Token, pos: Position) -> bool {
    ty.range().contains(pos) && ty.start().character != pos.character
}

fn make_items(items: &mut Vec<Item>, mut range: Range) {
    range.start.character += 1;
    for ty in &LANGUAGE_DATA.entry_types {
        let item = Item::new(range, ItemData::EntryType { ty });
        items.push(item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;

    #[tokio::test]
    async fn empty_latex_document() {
        let req = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_bibtex_entry_types(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let req = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_bibtex_entry_types(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn before_at_sign() {
        let req = FeatureTester::new()
            .file("main.bib", "@")
            .main("main.bib")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_bibtex_entry_types(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn after_at_sign() {
        let req = FeatureTester::new()
            .file("main.bib", "@")
            .main("main.bib")
            .position(0, 1)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_bibtex_entry_types(&req, &mut actual_items).await;

        assert!(!actual_items.is_empty());
        assert_eq!(actual_items[0].range, Range::new_simple(0, 1, 0, 1));
    }

    #[tokio::test]
    async fn inside_entry_type() {
        let req = FeatureTester::new()
            .file("main.bib", "@foo")
            .main("main.bib")
            .position(0, 2)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_bibtex_entry_types(&req, &mut actual_items).await;

        assert!(!actual_items.is_empty());
        assert_eq!(actual_items[0].range, Range::new_simple(0, 1, 0, 4));
    }

    #[tokio::test]
    async fn inside_entry_key() {
        let req = FeatureTester::new()
            .file("main.bib", "@article{foo,}")
            .main("main.bib")
            .position(0, 11)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_bibtex_entry_types(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn inside_comments() {
        let req = FeatureTester::new()
            .file("main.bib", "foo")
            .main("main.bib")
            .position(0, 2)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_bibtex_entry_types(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }
}
