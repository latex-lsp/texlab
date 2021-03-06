use cancellation::CancellationToken;
use lsp_types::{Hover, HoverContents, HoverParams, MarkupContent};

use crate::{features::cursor::CursorContext, LineIndexExt, LANGUAGE_DATA};

pub fn find_entry_type_hover(
    context: &CursorContext<HoverParams>,
    _token: &CancellationToken,
) -> Option<Hover> {
    let name = context
        .cursor
        .as_bibtex()
        .filter(|token| token.kind().is_type())?;

    let docs = LANGUAGE_DATA.entry_type_documentation(&name.text()[1..])?;
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: lsp_types::MarkupKind::Markdown,
            value: docs.to_string(),
        }),
        range: Some(
            context
                .request
                .main_document()
                .line_index
                .line_col_lsp_range(name.text_range()),
        ),
    })
}

#[cfg(test)]
mod tests {
    use lsp_types::{MarkupKind, Range};

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
            .hover();

        let context = CursorContext::new(request);
        let actual_hover = find_entry_type_hover(&context, CancellationToken::none());

        assert_eq!(actual_hover, None);
    }

    #[test]
    fn test_empty_bibtex_document() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "")])
            .main("main.bib")
            .line(0)
            .character(0)
            .build()
            .hover();

        let context = CursorContext::new(request);
        let actual_hover = find_entry_type_hover(&context, CancellationToken::none());

        assert_eq!(actual_hover, None);
    }

    #[test]
    fn test_known_entry_type() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", r#"@article{foo,}"#)])
            .main("main.bib")
            .line(0)
            .character(3)
            .build()
            .hover();

        let context = CursorContext::new(request);
        let actual_hover = find_entry_type_hover(&context, CancellationToken::none()).unwrap();

        let expected_hover = Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: LANGUAGE_DATA
                    .entry_type_documentation("article")
                    .unwrap()
                    .into(),
            }),
            range: Some(Range::new_simple(0, 0, 0, 8)),
        };
        assert_eq!(actual_hover, expected_hover);
    }

    #[test]
    fn test_unknown_field() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", r#"@foo{bar,}"#)])
            .main("main.bib")
            .line(0)
            .character(3)
            .build()
            .hover();

        let context = CursorContext::new(request);
        let actual_hover = find_entry_type_hover(&context, CancellationToken::none());

        assert_eq!(actual_hover, None);
    }

    #[test]
    fn test_entry_key() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", r#"@article{foo,}"#)])
            .main("main.bib")
            .line(0)
            .character(11)
            .build()
            .hover();

        let context = CursorContext::new(request);
        let actual_hover = find_entry_type_hover(&context, CancellationToken::none());

        assert_eq!(actual_hover, None);
    }
}
