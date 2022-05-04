use lsp_types::{Location, ReferenceParams};
use rowan::ast::AstNode;

use crate::{
    features::cursor::CursorContext,
    syntax::bibtex::{self, HasKey},
    LineIndexExt,
};

pub fn find_string_references(
    context: &CursorContext<ReferenceParams>,
    items: &mut Vec<Location>,
) -> Option<()> {
    let name_text = context
        .cursor
        .as_bibtex()
        .filter(|token| {
            let parent = token.parent().unwrap();
            (token.kind() == bibtex::WORD && bibtex::Value::can_cast(parent.kind()))
                || (token.kind() == bibtex::KEY && bibtex::StringDef::can_cast(parent.kind()))
        })?
        .text();

    let document = context.request.main_document();
    let data = document.data.as_bibtex()?;
    for node in bibtex::SyntaxNode::new_root(data.green.clone()).descendants() {
        if let Some(name) = bibtex::StringDef::cast(node.clone())
            .and_then(|string| string.key())
            .filter(|name| {
                context.request.params.context.include_declaration && name.text() == name_text
            })
            .or_else(|| {
                bibtex::Value::cast(node)
                    .and_then(|token| token.syntax().first_token())
                    .filter(|name| name.text() == name_text)
            })
        {
            items.push(Location::new(
                document.uri.as_ref().clone(),
                document.line_index.line_col_lsp_range(name.text_range()),
            ));
        }
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use lsp_types::Range;

    use crate::{features::testing::FeatureTester, RangeExt};

    use super::*;

    #[test]
    fn test_definition() {
        let tester = FeatureTester::builder()
            .files(vec![(
                "main.bib",
                indoc! { r#"
                    @string{foo = {Foo}}
                    @string{bar = {Bar}}
                    @article{baz, author = foo}
                "# },
            )])
            .main("main.bib")
            .line(2)
            .character(24)
            .build();
        let uri = tester.uri("main.bib");

        let mut actual_references = Vec::new();
        let request = tester.reference();
        let context = CursorContext::new(request);
        find_string_references(&context, &mut actual_references);

        let expected_references = vec![Location::new(
            uri.as_ref().clone(),
            Range::new_simple(2, 23, 2, 26),
        )];
        assert_eq!(actual_references, expected_references);
    }

    #[test]
    fn test_definition_include_declaration() {
        let tester = FeatureTester::builder()
            .files(vec![(
                "main.bib",
                indoc! { r#"
                    @string{foo = {Foo}}
                    @string{bar = {Bar}}
                    @article{baz, author = foo}
                "# },
            )])
            .main("main.bib")
            .line(2)
            .character(24)
            .include_declaration(true)
            .build();
        let uri = tester.uri("main.bib");

        let mut actual_references = Vec::new();
        let request = tester.reference();
        let context = CursorContext::new(request);
        find_string_references(&context, &mut actual_references);

        let expected_references = vec![
            Location::new(uri.as_ref().clone(), Range::new_simple(0, 8, 0, 11)),
            Location::new(uri.as_ref().clone(), Range::new_simple(2, 23, 2, 26)),
        ];
        assert_eq!(actual_references, expected_references);
    }

    #[test]
    fn test_reference() {
        let tester = FeatureTester::builder()
            .files(vec![(
                "main.bib",
                indoc! { r#"
                    @string{foo = {Foo}}
                    @string{bar = {Bar}}
                    @article{baz, author = foo}
                "# },
            )])
            .main("main.bib")
            .line(0)
            .character(10)
            .build();
        let uri = tester.uri("main.bib");

        let mut actual_references = Vec::new();
        let request = tester.reference();
        let context = CursorContext::new(request);
        find_string_references(&context, &mut actual_references);

        let expected_references = vec![Location::new(
            uri.as_ref().clone(),
            Range::new_simple(2, 23, 2, 26),
        )];
        assert_eq!(actual_references, expected_references);
    }

    #[test]
    fn test_reference_include_declaration() {
        let tester = FeatureTester::builder()
            .files(vec![(
                "main.bib",
                indoc! { r#"
                    @string{foo = {Foo}}
                    @string{bar = {Bar}}
                    @article{baz, author = foo}
                "# },
            )])
            .main("main.bib")
            .line(0)
            .character(10)
            .include_declaration(true)
            .build();
        let uri = tester.uri("main.bib");

        let mut actual_references = Vec::new();
        let request = tester.reference();
        let context = CursorContext::new(request);
        find_string_references(&context, &mut actual_references);

        let expected_references = vec![
            Location::new(uri.as_ref().clone(), Range::new_simple(0, 8, 0, 11)),
            Location::new(uri.as_ref().clone(), Range::new_simple(2, 23, 2, 26)),
        ];
        assert_eq!(actual_references, expected_references);
    }

    #[test]
    fn test_empty_latex_document() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "")])
            .main("main.tex")
            .line(0)
            .character(0)
            .build()
            .reference();

        let mut actual_references = Vec::new();
        let context = CursorContext::new(request);
        find_string_references(&context, &mut actual_references);

        assert!(actual_references.is_empty());
    }

    #[test]
    fn test_empty_bibtex_document() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "")])
            .main("main.bib")
            .line(0)
            .character(0)
            .build()
            .reference();

        let mut actual_references = Vec::new();
        let context = CursorContext::new(request);
        find_string_references(&context, &mut actual_references);

        assert!(actual_references.is_empty());
    }
}
