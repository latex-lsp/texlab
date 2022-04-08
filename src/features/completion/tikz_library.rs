use cancellation::CancellationToken;
use lsp_types::CompletionParams;
use rowan::ast::AstNode;

use crate::{features::cursor::CursorContext, syntax::latex, LANGUAGE_DATA};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_tikz_libraries<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
    cancellation_token: &CancellationToken,
) -> Option<()> {
    cancellation_token.result().ok()?;

    let (_, range, group) = context.find_curly_group_word_list()?;

    let import = latex::TikzLibraryImport::cast(group.syntax().parent()?)?;

    if import.command()?.text() == "\\usepgflibrary" {
        for name in &LANGUAGE_DATA.pgf_libraries {
            items.push(InternalCompletionItem::new(
                range,
                InternalCompletionItemData::PgfLibrary { name },
            ));
        }
    } else {
        for name in &LANGUAGE_DATA.tikz_libraries {
            items.push(InternalCompletionItem::new(
                range,
                InternalCompletionItemData::TikzLibrary { name },
            ));
        }
    }

    Some(())
}

#[cfg(test)]
mod tests {
    use rowan::TextRange;

    use crate::features::testing::FeatureTester;

    use super::*;

    #[test]
    fn test_empty_latex_document() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "")])
            .main("main.tex")
            .line(0)
            .character(0)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_tikz_libraries(&context, &mut actual_items, CancellationToken::none());

        assert!(actual_items.is_empty());
    }

    #[test]
    fn test_empty_bibtex_document() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "")])
            .main("main.bib")
            .line(0)
            .character(0)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_tikz_libraries(&context, &mut actual_items, CancellationToken::none());

        assert!(actual_items.is_empty());
    }

    #[test]
    fn test_latex_simple_pgf() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\usepgflibrary{}")])
            .main("main.tex")
            .line(0)
            .character(15)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_tikz_libraries(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(15.into(), 15.into()));
        }
    }

    #[test]
    fn test_latex_open_brace_pgf() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\usepgflibrary{")])
            .main("main.tex")
            .line(0)
            .character(15)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_tikz_libraries(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(15.into(), 15.into()));
        }
    }

    #[test]
    fn test_latex_simple_tikz() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\usetikzlibrary{}")])
            .main("main.tex")
            .line(0)
            .character(16)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_tikz_libraries(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(16.into(), 16.into()));
        }
    }

    #[test]
    fn test_latex_open_brace_tikz() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\usetikzlibrary{")])
            .main("main.tex")
            .line(0)
            .character(16)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_tikz_libraries(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(16.into(), 16.into()));
        }
    }
}
