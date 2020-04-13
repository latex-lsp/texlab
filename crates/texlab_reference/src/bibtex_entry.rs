use futures_boxed::boxed;
use texlab_feature::{DocumentContent, FeatureProvider, FeatureRequest};
use texlab_protocol::{Location, RangeExt, ReferenceParams, Url};
use texlab_syntax::{bibtex, latex, SyntaxNode};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct BibtexEntryReferenceProvider;

impl FeatureProvider for BibtexEntryReferenceProvider {
    type Params = ReferenceParams;
    type Output = Vec<Location>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut refs = Vec::new();
        if let Some(key) = Self::find_key(req) {
            for doc in req.related() {
                match &doc.content {
                    DocumentContent::Latex(table) => table
                        .citations
                        .iter()
                        .flat_map(|citation| citation.keys(&table))
                        .filter(|citation| citation.text() == key)
                        .map(|citation| Location::new(doc.uri.clone().into(), citation.range()))
                        .for_each(|location| refs.push(location)),
                    DocumentContent::Bibtex(tree) => {
                        if req.params.context.include_declaration {
                            let uri: Url = doc.uri.clone().into();
                            tree.children(tree.root)
                                .filter_map(|node| tree.as_entry(node))
                                .filter_map(|entry| entry.key.as_ref())
                                .filter(|key_tok| key_tok.text() == key)
                                .map(|key_tok| Location::new(uri.clone(), key_tok.range()))
                                .for_each(|location| refs.push(location));
                        }
                    }
                }
            }
        }
        refs
    }
}

impl BibtexEntryReferenceProvider {
    fn find_key(req: &FeatureRequest<ReferenceParams>) -> Option<&str> {
        let pos = req.params.text_document_position.position;
        match &req.current().content {
            DocumentContent::Latex(table) => table
                .citations
                .iter()
                .flat_map(|citation| citation.keys(&table))
                .find(|key| key.range().contains(pos))
                .map(latex::Token::text),
            DocumentContent::Bibtex(tree) => tree
                .children(tree.root)
                .filter_map(|node| tree.as_entry(node))
                .filter_map(|entry| entry.key.as_ref())
                .find(|key| key.range().contains(pos))
                .map(bibtex::Token::text),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use texlab_feature::FeatureTester;
    use texlab_protocol::Range;

    #[tokio::test]
    async fn entry() {
        let actual_refs = FeatureTester::new()
            .file("foo.bib", r#"@article{foo, bar = {baz}}"#)
            .file(
                "bar.tex",
                indoc!(
                    r#"
                        \addbibresource{foo.bib}
                        \cite{foo}
                    "#
                ),
            )
            .file("baz.tex", r#"\cite{foo}"#)
            .main("foo.bib")
            .position(0, 9)
            .test_reference(BibtexEntryReferenceProvider)
            .await;

        let expected_refs = vec![Location::new(
            FeatureTester::uri("bar.tex").into(),
            Range::new_simple(1, 6, 1, 9),
        )];

        assert_eq!(actual_refs, expected_refs);
    }

    #[tokio::test]
    async fn entry_include_declaration() {
        let actual_refs = FeatureTester::new()
            .file("foo.bib", r#"@article{foo, bar = {baz}}"#)
            .file(
                "bar.tex",
                indoc!(
                    r#"
                        \addbibresource{foo.bib}
                        \cite{foo}
                    "#
                ),
            )
            .file("baz.tex", r#"\cite{foo}"#)
            .main("foo.bib")
            .position(0, 9)
            .include_declaration()
            .test_reference(BibtexEntryReferenceProvider)
            .await;

        let expected_refs = vec![
            Location::new(
                FeatureTester::uri("foo.bib").into(),
                Range::new_simple(0, 9, 0, 12),
            ),
            Location::new(
                FeatureTester::uri("bar.tex").into(),
                Range::new_simple(1, 6, 1, 9),
            ),
        ];

        assert_eq!(actual_refs, expected_refs);
    }

    #[tokio::test]
    async fn citation() {
        let actual_refs = FeatureTester::new()
            .file("foo.bib", r#"@article{foo, bar = {baz}}"#)
            .file(
                "bar.tex",
                indoc!(
                    r#"
                        \addbibresource{foo.bib}
                        \cite{foo}
                    "#
                ),
            )
            .file("baz.tex", r#"\cite{foo}"#)
            .main("bar.tex")
            .position(1, 8)
            .test_reference(BibtexEntryReferenceProvider)
            .await;

        let expected_refs = vec![Location::new(
            FeatureTester::uri("bar.tex").into(),
            Range::new_simple(1, 6, 1, 9),
        )];

        assert_eq!(actual_refs, expected_refs);
    }

    #[tokio::test]
    async fn citation_include_declaration() {
        let actual_refs = FeatureTester::new()
            .file("foo.bib", r#"@article{foo, bar = {baz}}"#)
            .file(
                "bar.tex",
                indoc!(
                    r#"
                        \addbibresource{foo.bib}
                        \cite{foo}
                    "#
                ),
            )
            .file("baz.tex", r#"\cite{foo}"#)
            .main("bar.tex")
            .position(1, 8)
            .include_declaration()
            .test_reference(BibtexEntryReferenceProvider)
            .await;

        let expected_refs = vec![
            Location::new(
                FeatureTester::uri("bar.tex").into(),
                Range::new_simple(1, 6, 1, 9),
            ),
            Location::new(
                FeatureTester::uri("foo.bib").into(),
                Range::new_simple(0, 9, 0, 12),
            ),
        ];

        assert_eq!(actual_refs, expected_refs);
    }

    #[tokio::test]
    async fn empty_latex_document() {
        let actual_refs = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_reference(BibtexEntryReferenceProvider)
            .await;

        assert!(actual_refs.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_refs = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_reference(BibtexEntryReferenceProvider)
            .await;

        assert!(actual_refs.is_empty());
    }
}
