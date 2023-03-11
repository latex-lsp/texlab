use lsp_types::{FoldingRange, FoldingRangeKind, Range, Url};
use rowan::ast::AstNode;

use crate::{
    db::{parse::DocumentData, Workspace},
    syntax::{bibtex, latex},
    util::line_index_ext::LineIndexExt,
    Db,
};

pub fn find_all(db: &dyn Db, uri: &Url) -> Option<Vec<FoldingRange>> {
    let document = Workspace::get(db).lookup_uri(db, uri)?;
    let line_index = document.line_index(db);
    let foldings = match document.parse(db) {
        DocumentData::Tex(data) => {
            let mut results = Vec::new();
            let root = data.root(db);
            for node in root.descendants() {
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
            let root = data.root(db);
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
        DocumentData::Log(_) | DocumentData::TexlabRoot(_) | DocumentData::Tectonic(_) => {
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
