use cancellation::CancellationToken;
use lsp_types::{Hover, HoverContents, HoverParams};

use crate::{
    citation,
    features::FeatureRequest,
    syntax::{bibtex, latex, CstNode},
    DocumentData, LineIndexExt,
};

pub fn find_citation_hover(
    request: &FeatureRequest<HoverParams>,
    _token: &CancellationToken,
) -> Option<Hover> {
    let main_document = request.main_document();

    let offset = main_document
        .line_index
        .offset_lsp(request.params.text_document_position_params.position);

    let (key, key_range) = match &main_document.data {
        DocumentData::Latex(data) => {
            let word = data
                .root
                .token_at_offset(offset)
                .right_biased()
                .filter(|token| token.kind() == latex::WORD)?;

            latex::Citation::cast(word.parent().parent()?);
            (word.text().to_string(), word.text_range())
        }
        DocumentData::Bibtex(data) => {
            let word = data
                .root
                .token_at_offset(offset)
                .right_biased()
                .filter(|token| token.kind() == bibtex::WORD)?;

            bibtex::Entry::cast(word.parent())?;
            (word.text().to_string(), word.text_range())
        }
        DocumentData::BuildLog(_) => return None,
    };

    let contents = request.subset.documents.iter().find_map(|document| {
        document
            .data
            .as_bibtex()
            .and_then(|data| citation::render_citation(&data.root, &key))
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

        let actual_hover = find_citation_hover(&request, CancellationToken::none());

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

        let actual_hover = find_citation_hover(&request, CancellationToken::none());

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

        let actual_hover = find_citation_hover(&request, CancellationToken::none()).unwrap();

        let expected_hover = Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Bar, F. (1337). *Baz Qux*.".into(),
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

        let actual_hover = find_citation_hover(&request, CancellationToken::none()).unwrap();

        let expected_hover = Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "Bar, F. (1337). *Baz Qux*.".into(),
            }),
            range: Some(Range::new_simple(0, 9, 0, 12)),
        };
        assert_eq!(actual_hover, expected_hover);
    }
}
