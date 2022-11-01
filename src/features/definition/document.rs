use rowan::TextRange;

use crate::util::cursor::CursorContext;

use super::DefinitionResult;

pub(super) fn goto_document_definition(context: &CursorContext) -> Option<Vec<DefinitionResult>> {
    let db = context.db;
    context
        .workspace
        .parents(db, context.distro, context.document)
        .iter()
        .map(|&parent| context.workspace.graph(db, parent, context.distro))
        .flat_map(|graph| graph.edges(db))
        .filter(|edge| edge.source(db) == context.document)
        .find_map(|edge| {
            let range = edge.origin(db).into_explicit()?.link.range(db);
            if range.contains_inclusive(context.offset) {
                Some(vec![DefinitionResult {
                    origin_selection_range: range,
                    target: edge.target(db)?,
                    target_range: TextRange::default(),
                    target_selection_range: TextRange::default(),
                }])
            } else {
                None
            }
        })
}
