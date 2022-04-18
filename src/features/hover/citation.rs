use lsp_types::{Hover, HoverContents, HoverParams};

use crate::{citation, features::cursor::CursorContext, syntax::bibtex, LineIndexExt};

pub fn find_citation_hover(context: &CursorContext<HoverParams>) -> Option<Hover> {
    let main_document = context.request.main_document();

    let (key_text, key_range) = context
        .find_citation_key_word()
        .or_else(|| context.find_citation_key_command())
        .or_else(|| context.find_entry_key())?;

    let contents = context
        .request
        .workspace
        .documents_by_uri
        .values()
        .find_map(|document| {
            document.data.as_bibtex().and_then(|data| {
                citation::render_citation(
                    &bibtex::SyntaxNode::new_root(data.green.clone()),
                    &key_text,
                )
            })
        })?;

    Some(Hover {
        range: Some(main_document.line_index.line_col_lsp_range(key_range)),
        contents: HoverContents::Markup(contents),
    })
}

#[cfg(test)]
mod tests {
    use lsp_types::{MarkupContent, MarkupKind, Range};

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
        let actual_hover = find_citation_hover(&context);

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
        let actual_hover = find_citation_hover(&context);

        assert_eq!(actual_hover, None);
    }

    #[test]
    fn test_inside_cite() {
        let request = FeatureTester::builder()
            .files(vec![
                (
                    "main.bib",
                    "@article{foo, author = {Foo Bar}, title = {Baz Qux}, year = 1337}",
                ),
                ("main.tex", "\\addbibresource{main.bib}\n\\cite{foo}"),
            ])
            .main("main.tex")
            .line(1)
            .character(7)
            .build()
            .hover();

        let context = CursorContext::new(request);
        let actual_hover = find_citation_hover(&context).unwrap();

        let expected_hover = Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Bar, Foo. (1337). *Baz Qux*.".into(),
            }),
            range: Some(Range::new_simple(1, 6, 1, 9)),
        };
        assert_eq!(actual_hover, expected_hover);
    }

    #[test]
    fn test_inside_entry() {
        let request = FeatureTester::builder()
            .files(vec![
                (
                    "main.bib",
                    "@article{foo, author = {Foo Bar}, title = {Baz Qux}, year = 1337}",
                ),
                ("main.tex", "\\addbibresource{main.bib}\n\\cite{foo}"),
            ])
            .main("main.bib")
            .line(0)
            .character(11)
            .build()
            .hover();

        let context = CursorContext::new(request);
        let actual_hover = find_citation_hover(&context).unwrap();

        let expected_hover = Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Bar, Foo. (1337). *Baz Qux*.".into(),
            }),
            range: Some(Range::new_simple(0, 9, 0, 12)),
        };
        assert_eq!(actual_hover, expected_hover);
    }
}
