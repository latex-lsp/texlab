use futures_boxed::boxed;
use texlab_feature::{DocumentContent, FeatureProvider, FeatureRequest};
use texlab_protocol::{Location, RangeExt, ReferenceParams};
use texlab_syntax::{latex, LatexLabelKind, SyntaxNode};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexLabelReferenceProvider;

impl FeatureProvider for LatexLabelReferenceProvider {
    type Params = ReferenceParams;
    type Output = Vec<Location>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut refs = Vec::new();
        if let Some(def) = Self::find_name(req) {
            for doc in req.related() {
                if let DocumentContent::Latex(table) = &doc.content {
                    table
                        .labels
                        .iter()
                        .filter(|label| Self::is_included(req, label))
                        .flat_map(|label| label.names(&table))
                        .filter(|label| label.text() == def)
                        .map(|label| Location::new(doc.uri.clone().into(), label.range()))
                        .for_each(|location| refs.push(location));
                }
            }
        }
        refs
    }
}

impl LatexLabelReferenceProvider {
    fn find_name(req: &FeatureRequest<ReferenceParams>) -> Option<&str> {
        let pos = req.params.text_document_position.position;
        if let DocumentContent::Latex(table) = &req.current().content {
            table
                .labels
                .iter()
                .flat_map(|label| label.names(&table))
                .find(|label| label.range().contains(pos))
                .map(latex::Token::text)
        } else {
            None
        }
    }

    fn is_included(req: &FeatureRequest<ReferenceParams>, label: &latex::Label) -> bool {
        match label.kind {
            LatexLabelKind::Reference(_) => true,
            LatexLabelKind::Definition => req.params.context.include_declaration,
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
    async fn definition() {
        let actual_refs = FeatureTester::new()
            .file("foo.tex", r#"\label{foo}"#)
            .file(
                "bar.tex",
                indoc!(
                    r#"
                        \input{foo.tex}
                        \ref{foo}
                    "#
                ),
            )
            .file("baz.tex", r#"\ref{foo}"#)
            .main("foo.tex")
            .position(0, 8)
            .test_reference(LatexLabelReferenceProvider)
            .await;

        let expected_refs = vec![Location::new(
            FeatureTester::uri("bar.tex").into(),
            Range::new_simple(1, 5, 1, 8),
        )];

        assert_eq!(actual_refs, expected_refs);
    }

    #[tokio::test]
    async fn definition_include_declaration() {
        let actual_refs = FeatureTester::new()
            .file("foo.tex", r#"\label{foo}"#)
            .file(
                "bar.tex",
                indoc!(
                    r#"
                        \input{foo.tex}
                        \ref{foo}
                    "#
                ),
            )
            .file("baz.tex", r#"\ref{foo}"#)
            .main("foo.tex")
            .position(0, 8)
            .include_declaration()
            .test_reference(LatexLabelReferenceProvider)
            .await;

        let expected_refs = vec![
            Location::new(
                FeatureTester::uri("foo.tex").into(),
                Range::new_simple(0, 7, 0, 10),
            ),
            Location::new(
                FeatureTester::uri("bar.tex").into(),
                Range::new_simple(1, 5, 1, 8),
            ),
        ];

        assert_eq!(actual_refs, expected_refs);
    }

    #[tokio::test]
    async fn reference() {
        let actual_refs = FeatureTester::new()
            .file("foo.tex", r#"\label{foo}"#)
            .file(
                "bar.tex",
                indoc!(
                    r#"
                        \input{foo.tex}
                        \ref{foo}
                    "#
                ),
            )
            .file("baz.tex", r#"\ref{foo}"#)
            .main("bar.tex")
            .position(1, 7)
            .test_reference(LatexLabelReferenceProvider)
            .await;

        let expected_refs = vec![Location::new(
            FeatureTester::uri("bar.tex").into(),
            Range::new_simple(1, 5, 1, 8),
        )];

        assert_eq!(actual_refs, expected_refs);
    }

    #[tokio::test]
    async fn reference_include_declaration() {
        let actual_refs = FeatureTester::new()
            .file("foo.tex", r#"\label{foo}"#)
            .file(
                "bar.tex",
                indoc!(
                    r#"
                        \input{foo.tex}
                        \ref{foo}
                    "#
                ),
            )
            .file("baz.tex", r#"\ref{foo}"#)
            .main("bar.tex")
            .position(1, 7)
            .include_declaration()
            .test_reference(LatexLabelReferenceProvider)
            .await;

        let expected_refs = vec![
            Location::new(
                FeatureTester::uri("bar.tex").into(),
                Range::new_simple(1, 5, 1, 8),
            ),
            Location::new(
                FeatureTester::uri("foo.tex").into(),
                Range::new_simple(0, 7, 0, 10),
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
            .test_reference(LatexLabelReferenceProvider)
            .await;

        assert!(actual_refs.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document() {
        let actual_refs = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_reference(LatexLabelReferenceProvider)
            .await;

        assert!(actual_refs.is_empty());
    }
}
