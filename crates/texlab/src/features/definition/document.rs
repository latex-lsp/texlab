use rowan::TextRange;

use crate::util::cursor::CursorContext;

use super::DefinitionResult;

pub(super) fn goto_definition<'a>(
    context: &CursorContext<'a>,
) -> Option<Vec<DefinitionResult<'a>>> {
    context
        .workspace
        .parents(context.document)
        .iter()
        .copied()
        .chain(std::iter::once(context.document))
        .flat_map(|parent| base_db::graph::Graph::new(context.workspace, parent).edges)
        .filter(|edge| edge.source == context.document)
        .find_map(|edge| {
            let range = edge.weight?.link.path.range;
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
