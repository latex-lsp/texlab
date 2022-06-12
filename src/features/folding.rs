use lsp_types::{FoldingRange, FoldingRangeKind, FoldingRangeParams, Range};
use rowan::ast::AstNode;

use crate::{
    syntax::{bibtex, latex},
    DocumentData, LineIndexExt,
};

use super::FeatureRequest;

pub fn find_foldings(request: FeatureRequest<FoldingRangeParams>) -> Vec<FoldingRange> {
    let mut foldings = Vec::new();
    let main_document = request.main_document();
    match &main_document.data {
        DocumentData::Latex(data) => {
            for node in latex::SyntaxNode::new_root(data.green.clone()).descendants() {
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
            for node in bibtex::SyntaxNode::new_root(data.green.clone()).descendants() {
                if matches!(
                    node.kind(),
                    bibtex::PREAMBLE | bibtex::STRING | bibtex::ENTRY
                ) {
                    foldings.push(create_range(
                        main_document
                            .line_index
                            .line_col_lsp_range(node.text_range()),
                    ));
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
