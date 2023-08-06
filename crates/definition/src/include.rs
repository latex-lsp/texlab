use rowan::TextRange;

use crate::DefinitionContext;

use super::DefinitionResult;

pub(super) fn goto_definition(context: &mut DefinitionContext) -> Option<()> {
    let start = context.params.document;
    let parents = context.params.workspace.parents(start);
    let results = parents
        .into_iter()
        .chain(std::iter::once(start))
        .flat_map(|parent| base_db::graph::Graph::new(context.params.workspace, parent).edges)
        .filter(|edge| edge.source == start)
        .flat_map(|edge| {
            let origin_selection_range = edge.weight?.link.path.range;
            if origin_selection_range.contains_inclusive(context.params.offset) {
                Some(DefinitionResult {
                    origin_selection_range,
                    target: edge.target,
                    target_range: TextRange::default(),
                    target_selection_range: TextRange::default(),
                })
            } else {
                None
            }
        });

    context.results.extend(results);
    Some(())
}
