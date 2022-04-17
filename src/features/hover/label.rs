use lsp_types::{Hover, HoverContents, HoverParams};

use crate::{features::cursor::CursorContext, render_label, LineIndexExt};

pub fn find_label_hover(context: &CursorContext<HoverParams>) -> Option<Hover> {
    let main_document = context.request.main_document();

    let (name_text, name_range) = context
        .find_label_name_key()
        .or_else(|| context.find_label_name_command())?;

    let label = render_label(&context.request.subset, &name_text, None)?;

    Some(Hover {
        range: Some(main_document.line_index.line_col_lsp_range(name_range)),
        contents: HoverContents::Markup(label.documentation()),
    })
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
            .hover();

        let context = CursorContext::new(request);
        let actual_hover = find_label_hover(&context);

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
        let actual_hover = find_label_hover(&context);

        assert_eq!(actual_hover, None);
    }

    #[test]
    fn test_section() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", r#"\section{Foo}\label{sec:foo}"#)])
            .main("main.tex")
            .line(0)
            .character(23)
            .build()
            .hover();

        let context = CursorContext::new(request);
        let actual_hover = find_label_hover(&context).unwrap();

        assert_eq!(actual_hover.range.unwrap(), Range::new_simple(0, 20, 0, 27));
    }
}
