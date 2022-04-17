use lsp_types::{Location, ReferenceParams};
use rowan::ast::AstNode;

use crate::{
    features::cursor::CursorContext,
    syntax::{bibtex, latex},
    DocumentData, LineIndexExt,
};

pub fn find_entry_references(
    context: &CursorContext<ReferenceParams>,
    references: &mut Vec<Location>,
) -> Option<()> {
    let (key_text, _) = context
        .find_citation_key_word()
        .or_else(|| context.find_citation_key_command())
        .or_else(|| context.find_entry_key())?;

    for document in &context.request.subset.documents {
        match &document.data {
            DocumentData::Latex(data) => {
                latex::SyntaxNode::new_root(data.green.clone())
                    .descendants()
                    .filter_map(latex::Citation::cast)
                    .filter_map(|citation| citation.key_list())
                    .flat_map(|keys| keys.keys())
                    .filter(|key| key.to_string() == key_text)
                    .map(|key| {
                        document
                            .line_index
                            .line_col_lsp_range(latex::small_range(&key))
                    })
                    .for_each(|range| {
                        references.push(Location::new(document.uri.as_ref().clone().into(), range));
                    });
            }
            DocumentData::Bibtex(data) if context.request.params.context.include_declaration => {
                bibtex::SyntaxNode::new_root(data.green.clone())
                    .children()
                    .filter_map(bibtex::Entry::cast)
                    .filter_map(|entry| entry.key())
                    .filter(|key| key.to_string() == key_text)
                    .map(|key| {
                        document
                            .line_index
                            .line_col_lsp_range(bibtex::small_range(&key))
                    })
                    .for_each(|range| {
                        references.push(Location::new(document.uri.as_ref().clone().into(), range));
                    });
            }
            DocumentData::Bibtex(_) | DocumentData::BuildLog(_) => {}
        }
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use lsp_types::Range;

    use crate::{features::testing::FeatureTester, RangeExt};

    use super::*;

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
        find_entry_references(&context, &mut actual_references);

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
        find_entry_references(&context, &mut actual_references);

        assert!(actual_references.is_empty());
    }

    #[test]
    fn test_definition() {
        let tester = FeatureTester::builder()
            .files(vec![
                ("foo.bib", r#"@article{foo,}"#),
                ("bar.tex", r#"\cite{foo}\addbibresource{foo.bib}"#),
            ])
            .main("foo.bib")
            .line(0)
            .character(11)
            .build();
        let uri = tester.uri("bar.tex");
        let mut actual_references = Vec::new();

        let request = tester.reference();
        let context = CursorContext::new(request);
        find_entry_references(&context, &mut actual_references);

        let expected_references = vec![Location::new(
            uri.as_ref().clone().into(),
            Range::new_simple(0, 6, 0, 9),
        )];
        assert_eq!(actual_references, expected_references);
    }

    #[test]
    fn test_definition_include_declaration() {
        let tester = FeatureTester::builder()
            .files(vec![
                ("foo.bib", r#"@article{foo,}"#),
                ("bar.tex", r#"\cite{foo}\addbibresource{foo.bib}"#),
            ])
            .main("foo.bib")
            .line(0)
            .character(11)
            .include_declaration(true)
            .build();
        let uri1 = tester.uri("foo.bib");
        let uri2 = tester.uri("bar.tex");
        let mut actual_references = Vec::new();

        let request = tester.reference();
        let context = CursorContext::new(request);
        find_entry_references(&context, &mut actual_references);

        let expected_references = vec![
            Location::new(uri1.as_ref().clone().into(), Range::new_simple(0, 9, 0, 12)),
            Location::new(uri2.as_ref().clone().into(), Range::new_simple(0, 6, 0, 9)),
        ];
        assert_eq!(actual_references, expected_references);
    }

    #[test]
    fn test_reference() {
        let tester = FeatureTester::builder()
            .files(vec![
                ("foo.bib", r#"@article{foo,}"#),
                ("bar.tex", r#"\cite{foo}\addbibresource{foo.bib}"#),
            ])
            .main("bar.tex")
            .line(0)
            .character(8)
            .build();
        let uri = tester.uri("bar.tex");
        let mut actual_references = Vec::new();

        let request = tester.reference();
        let context = CursorContext::new(request);
        find_entry_references(&context, &mut actual_references);

        let expected_references = vec![Location::new(
            uri.as_ref().clone().into(),
            Range::new_simple(0, 6, 0, 9),
        )];
        assert_eq!(actual_references, expected_references);
    }

    #[test]
    fn test_reference_include_declaration() {
        let tester = FeatureTester::builder()
            .files(vec![
                ("foo.bib", r#"@article{foo,}"#),
                ("bar.tex", r#"\cite{foo}\addbibresource{foo.bib}"#),
            ])
            .main("bar.tex")
            .line(0)
            .character(6)
            .include_declaration(true)
            .build();
        let uri1 = tester.uri("foo.bib");
        let uri2 = tester.uri("bar.tex");
        let mut actual_references = Vec::new();

        let request = tester.reference();
        let context = CursorContext::new(request);
        find_entry_references(&context, &mut actual_references);

        let expected_references = vec![
            Location::new(uri2.as_ref().clone().into(), Range::new_simple(0, 6, 0, 9)),
            Location::new(uri1.as_ref().clone().into(), Range::new_simple(0, 9, 0, 12)),
        ];
        assert_eq!(actual_references, expected_references);
    }
}
