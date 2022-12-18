use rowan::TextRange;

use crate::{db::dependency_graph, util::cursor::CursorContext};

use super::DefinitionResult;

pub(super) fn goto_definition(context: &CursorContext) -> Option<Vec<DefinitionResult>> {
    let db = context.db;
    context
        .workspace
        .parents(db, context.document)
        .iter()
        .copied()
        .chain(std::iter::once(context.document))
        .flat_map(|parent| dependency_graph(db, parent).edges)
        .filter(|edge| edge.source == context.document)
        .find_map(|edge| {
            let range = edge.origin?.link.range(db);
            if range.contains_inclusive(context.offset) {
                Some(vec![DefinitionResult {
                    origin_selection_range: range,
                    target: edge.target,
                    target_range: TextRange::default(),
                    target_selection_range: TextRange::default(),
                }])
            } else {
                None
            }
        })
}
