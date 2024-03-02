use base_db::deps;
use rowan::TextRange;

use crate::DefinitionContext;

use super::DefinitionResult;

pub(super) fn goto_definition(context: &mut DefinitionContext) -> Option<()> {
    let feature = &context.params.feature;
    let start = feature.document;
    let parents = deps::parents(feature.workspace, start);
    let results = parents
        .into_iter()
        .chain(std::iter::once(start))
        .flat_map(|parent| deps::Graph::new(feature.workspace, parent).edges)
        .filter(|edge| edge.source == start)
        .flat_map(|edge| {
            let deps::EdgeData::DirectLink(data) = edge.data else {
                return None;
            };

            let origin_selection_range = data.link.path.range;
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
