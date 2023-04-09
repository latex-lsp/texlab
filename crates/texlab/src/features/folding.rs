use base_db::{DocumentData, Workspace};
use lsp_types::{FoldingRange, FoldingRangeKind, Range, Url};
use rowan::ast::AstNode;
use syntax::{bibtex, latex};

use crate::util::line_index_ext::LineIndexExt;

pub fn find_all(workspace: &Workspace, uri: &Url) -> Option<Vec<FoldingRange>> {
    let document = workspace.lookup(uri)?;
    let line_index = &document.line_index;
    let foldings = match &document.data {
        DocumentData::Tex(data) => {
            let mut results = Vec::new();
            for node in data.root_node().descendants() {
                if let Some(folding) = latex::Environment::cast(node.clone())
                    .map(|node| latex::small_range(&node))
                    .or_else(|| {
                        latex::Section::cast(node.clone()).map(|node| latex::small_range(&node))
                    })
                    .or_else(|| latex::EnumItem::cast(node).map(|node| latex::small_range(&node)))
                    .map(|node| line_index.line_col_lsp_range(node))
                    .map(create_range)
                {
                    results.push(folding);
                }
            }

            results
        }
        DocumentData::Bib(data) => {
            let root = data.root_node();
            root.descendants()
                .filter(|node| {
                    matches!(
                        node.kind(),
                        bibtex::PREAMBLE | bibtex::STRING | bibtex::ENTRY
                    )
                })
                .map(|node| create_range(line_index.line_col_lsp_range(node.text_range())))
                .collect()
        }
        DocumentData::Aux(_)
        | DocumentData::Log(_)
        | DocumentData::Root
        | DocumentData::Tectonic => {
            return None;
        }
    };

    Some(foldings)
}

fn create_range(range: Range) -> FoldingRange {
    FoldingRange {
        start_line: range.start.line,
        start_character: Some(range.start.character),
        end_line: range.end.line,
        end_character: Some(range.end.character),
        collapsed_text: None,
        kind: Some(FoldingRangeKind::Region),
    }
}
