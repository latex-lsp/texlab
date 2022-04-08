use cancellation::CancellationToken;
use lsp_types::CompletionParams;
use rowan::ast::AstNode;
use rustc_hash::FxHashSet;
use smol_str::SmolStr;

use crate::{component_db::COMPONENT_DATABASE, features::cursor::CursorContext, syntax::latex};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_imports<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
    cancellation_token: &CancellationToken,
) -> Option<()> {
    let (_, range, group) = context.find_curly_group_word_list()?;

    let (extension, mut factory): (
        &str,
        Box<dyn FnMut(SmolStr) -> InternalCompletionItemData<'a>>,
    ) = match group.syntax().parent()?.kind() {
        latex::PACKAGE_INCLUDE => (
            "sty",
            Box::new(|name| InternalCompletionItemData::Package { name }),
        ),
        latex::CLASS_INCLUDE => (
            "cls",
            Box::new(|name| InternalCompletionItemData::Class { name }),
        ),
        _ => return None,
    };

    let mut file_names = FxHashSet::default();
    for file_name in COMPONENT_DATABASE
        .components
        .iter()
        .flat_map(|comp| comp.file_names.iter())
        .filter(|file_name| file_name.ends_with(extension))
    {
        cancellation_token.result().ok()?;
        file_names.insert(file_name);
        let stem = &file_name[0..file_name.len() - 4];
        let data = factory(stem.into());
        let item = InternalCompletionItem::new(range, data);
        items.push(item);
    }

    let resolver = context.request.context.resolver.lock().unwrap();
    for file_name in resolver
        .files_by_name
        .keys()
        .filter(|file_name| file_name.ends_with(extension) && !file_names.contains(file_name))
    {
        cancellation_token.result().ok()?;

        let stem = &file_name[0..file_name.len() - 4];
        let data = factory(stem.into());
        let item = InternalCompletionItem::new(range, data);
        items.push(item);
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
        complete_imports(&context, &mut actual_items, CancellationToken::none());

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
        complete_imports(&context, &mut actual_items, CancellationToken::none());

        assert!(actual_items.is_empty());
    }

    #[test]
    fn test_latex_simple_package() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\usepackage{}")])
            .main("main.tex")
            .line(0)
            .character(12)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_imports(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(12.into(), 12.into()));
        }
    }

    #[test]
    fn test_latex_open_brace_package() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\usepackage{ \\foo")])
            .main("main.tex")
            .line(0)
            .character(12)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_imports(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(12.into(), 12.into()));
        }
    }

    #[test]
    fn test_latex_simple_class() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\documentclass{}")])
            .main("main.tex")
            .line(0)
            .character(15)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_imports(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(15.into(), 15.into()));
        }
    }

    #[test]
    fn test_latex_open_brace_class() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\documentclass{ \\foo")])
            .main("main.tex")
            .line(0)
            .character(15)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_imports(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(15.into(), 15.into()));
        }
    }
}
