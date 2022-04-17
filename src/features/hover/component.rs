use lsp_types::{Hover, HoverContents, HoverParams};

use crate::{
    component_db::COMPONENT_DATABASE, features::cursor::CursorContext, syntax::latex, LineIndexExt,
};

pub fn find_component_hover(context: &CursorContext<HoverParams>) -> Option<Hover> {
    let main_document = context.request.main_document();
    let data = main_document.data.as_latex()?;
    for link in &data.extras.explicit_links {
        if matches!(
            link.kind,
            latex::ExplicitLinkKind::Package | latex::ExplicitLinkKind::Class
        ) && link.stem_range.contains_inclusive(context.offset)
        {
            let docs = COMPONENT_DATABASE.documentation(&link.stem)?;
            return Some(Hover {
                contents: HoverContents::Markup(docs),
                range: Some(main_document.line_index.line_col_lsp_range(link.stem_range)),
            });
        }
    }
    None
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
        let actual_hover = find_component_hover(&context);

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
        let actual_hover = find_component_hover(&context);

        assert_eq!(actual_hover, None);
    }

    #[test]
    fn test_known_package() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", r#"\usepackage{amsmath}"#)])
            .main("main.tex")
            .line(0)
            .character(15)
            .build()
            .hover();

        let context = CursorContext::new(request);
        let actual_hover = find_component_hover(&context).unwrap();

        assert_eq!(actual_hover.range.unwrap(), Range::new_simple(0, 12, 0, 19));
    }

    #[test]
    fn test_unknown_class() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", r#"\documentclass{abcdefghijklmnop}"#)])
            .main("main.tex")
            .line(0)
            .character(20)
            .build()
            .hover();

        let context = CursorContext::new(request);
        let actual_hover = find_component_hover(&context);

        assert_eq!(actual_hover, None);
    }
}
