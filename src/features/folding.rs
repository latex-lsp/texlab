use cancellation::CancellationToken;
use lsp_types::{FoldingRange, FoldingRangeKind, FoldingRangeParams, Range};
use rowan::ast::AstNode;

use crate::{
    syntax::{bibtex, latex},
    DocumentData, LineIndexExt,
};

use super::FeatureRequest;

pub fn find_foldings(
    request: FeatureRequest<FoldingRangeParams>,
    token: &CancellationToken,
) -> Vec<FoldingRange> {
    let mut foldings = Vec::new();
    let main_document = request.main_document();
    match &main_document.data {
        DocumentData::Latex(data) => {
            for node in latex::SyntaxNode::new_root(data.root.clone()).descendants() {
                if token.is_canceled() {
                    break;
                }

                if let Some(folding) = latex::Environment::cast(node.clone())
                    .map(|node| latex::small_range(&node))
                    .or_else(|| {
                        latex::Section::cast(node.clone()).map(|node| latex::small_range(&node))
                    })
                    .or_else(|| latex::EnumItem::cast(node).map(|node| latex::small_range(&node)))
                    .map(|node| main_document.line_index.line_col_lsp_range(node))
                    .map(create_range)
                {
                    foldings.push(folding);
                }
            }
        }
        DocumentData::Bibtex(data) => {
            for node in bibtex::SyntaxNode::new_root(data.root.clone()).descendants() {
                if token.is_canceled() {
                    break;
                }

                if let Some(folding) = bibtex::Preamble::cast(node.clone())
                    .map(|node| bibtex::small_range(&node))
                    .or_else(|| {
                        bibtex::String::cast(node.clone()).map(|node| bibtex::small_range(&node))
                    })
                    .or_else(|| {
                        bibtex::Entry::cast(node.clone()).map(|node| bibtex::small_range(&node))
                    })
                    .map(|node| main_document.line_index.line_col_lsp_range(node))
                    .map(create_range)
                {
                    foldings.push(folding);
                }
            }
        }
        DocumentData::BuildLog(_) => {}
    }
    foldings
}

fn create_range(range: Range) -> FoldingRange {
    FoldingRange {
        start_line: range.start.line,
        start_character: Some(range.start.character),
        end_line: range.end.line,
        end_character: Some(range.end.character),
        kind: Some(FoldingRangeKind::Region),
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::features::testing::FeatureTester;

    use super::*;

    #[test]
    fn test_empty_latex_document() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "")])
            .main("main.tex")
            .build()
            .folding();

        let actual_foldings = find_foldings(request, CancellationToken::none());
        assert!(actual_foldings.is_empty());
    }

    #[test]
    fn test_empty_bibtex_document() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "")])
            .main("main.bib")
            .build()
            .folding();

        let actual_foldings = find_foldings(request, CancellationToken::none());
        assert!(actual_foldings.is_empty());
    }

    #[test]
    fn test_latex() {
        let req = FeatureTester::builder()
            .files(vec![(
                "main.tex",
                indoc! {r#"
                \begin{document}
                    \section{Foo}
                    foo
                    \subsection{Bar}
                    bar
                    \section{Baz}
                    baz
                    \section{Qux}
                \end{document}"# },
            )])
            .main("main.tex")
            .build()
            .folding();

        let mut actual_foldings = find_foldings(req, CancellationToken::none());
        actual_foldings.sort_by_key(|folding| (folding.start_line, folding.start_character));

        assert_eq!(actual_foldings.len(), 5);
        assert_eq!(
            actual_foldings[0],
            FoldingRange {
                start_line: 0,
                start_character: Some(0),
                end_line: 8,
                end_character: Some(14),
                kind: Some(FoldingRangeKind::Region)
            }
        );
        assert_eq!(
            actual_foldings[1],
            FoldingRange {
                start_line: 1,
                start_character: Some(4),
                end_line: 4,
                end_character: Some(7),
                kind: Some(FoldingRangeKind::Region)
            }
        );
        assert_eq!(
            actual_foldings[2],
            FoldingRange {
                start_line: 3,
                start_character: Some(4),
                end_line: 4,
                end_character: Some(7),
                kind: Some(FoldingRangeKind::Region)
            }
        );
        assert_eq!(
            actual_foldings[3],
            FoldingRange {
                start_line: 5,
                start_character: Some(4),
                end_line: 6,
                end_character: Some(7),
                kind: Some(FoldingRangeKind::Region)
            }
        );
        assert_eq!(
            actual_foldings[4],
            FoldingRange {
                start_line: 7,
                start_character: Some(4),
                end_line: 7,
                end_character: Some(17),
                kind: Some(FoldingRangeKind::Region)
            }
        );
    }

    #[test]
    fn test_bibtex() {
        let request = FeatureTester::builder()
            .files(vec![(
                "main.bib",
                indoc! {r#"
                some junk
                here

                @article{foo,
                    author = {bar},
                    title = {baz}
                }

                @string{foo = "bar"}

                @comment{foo,
                    author = {bar},
                    title = {baz}
                }

                @preamble{"foo"}"# },
            )])
            .main("main.bib")
            .build()
            .folding();

        let mut actual_foldings = find_foldings(request, CancellationToken::none());
        actual_foldings.sort_by_key(|folding| (folding.start_line, folding.start_character));

        assert_eq!(actual_foldings.len(), 3);
        assert_eq!(
            actual_foldings[0],
            FoldingRange {
                start_line: 3,
                start_character: Some(0),
                end_line: 6,
                end_character: Some(1),
                kind: Some(FoldingRangeKind::Region)
            }
        );
        assert_eq!(
            actual_foldings[1],
            FoldingRange {
                start_line: 8,
                start_character: Some(0),
                end_line: 8,
                end_character: Some(20),
                kind: Some(FoldingRangeKind::Region)
            }
        );
        assert_eq!(
            actual_foldings[2],
            FoldingRange {
                start_line: 15,
                start_character: Some(0),
                end_line: 15,
                end_character: Some(16),
                kind: Some(FoldingRangeKind::Region)
            }
        );
    }
}
